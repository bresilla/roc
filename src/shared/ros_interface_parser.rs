use anyhow::{anyhow, Result};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseType {
    Bool,
    Int8,
    UInt8,
    Int16,
    UInt16,
    Int32,
    UInt32,
    Int64,
    UInt64,
    Float32,
    Float64,
    String,
    WString,
    Char,
    WChar,
    Byte,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    Base(BaseType),
    Named(String), // pkg/msg/Type or builtin_interfaces/msg/Time
    Array(Box<FieldType>, usize),
    BoundedSequence(Box<FieldType>, usize),
    UnboundedSequence(Box<FieldType>),
    BoundedString(usize),
    BoundedWString(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub ty: FieldType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageSpec {
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceSpec {
    pub request: MessageSpec,
    pub response: MessageSpec,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionSpec {
    pub goal: MessageSpec,
    pub result: MessageSpec,
    pub feedback: MessageSpec,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterfaceSpec {
    Msg(MessageSpec),
    Srv(ServiceSpec),
    Action(ActionSpec),
}

fn strip_comment(line: &str) -> &str {
    match line.split_once('#') {
        Some((left, _)) => left,
        None => line,
    }
}

fn parse_base_type(s: &str) -> Option<BaseType> {
    Some(match s {
        "bool" | "boolean" => BaseType::Bool,
        "int8" => BaseType::Int8,
        "uint8" => BaseType::UInt8,
        "int16" => BaseType::Int16,
        "uint16" => BaseType::UInt16,
        "int32" => BaseType::Int32,
        "uint32" => BaseType::UInt32,
        "int64" => BaseType::Int64,
        "uint64" => BaseType::UInt64,
        "float32" => BaseType::Float32,
        "float64" | "double" => BaseType::Float64,
        "string" => BaseType::String,
        "wstring" => BaseType::WString,
        "char" => BaseType::Char,
        "wchar" => BaseType::WChar,
        "byte" | "octet" => BaseType::Byte,
        _ => return None,
    })
}

fn normalize_named_type(current_pkg: &str, token: &str, default_ns: &str) -> String {
    if token.contains('/') {
        return token.to_string();
    }
    // Unqualified nested types in .srv/.action sections refer to same package.
    format!("{}/{}/{}", current_pkg, default_ns, token)
}

fn parse_type_token(current_pkg: &str, default_ns: &str, token: &str) -> Result<FieldType> {
    // bounded string: string<=N or wstring<=N
    if let Some((base, bound)) = token.split_once("<=") {
        let n: usize = bound
            .parse()
            .map_err(|_| anyhow!("Invalid bound in type '{}'", token))?;
        return Ok(match base {
            "string" => FieldType::BoundedString(n),
            "wstring" => FieldType::BoundedWString(n),
            _ => return Err(anyhow!("Invalid bounded type '{}'", token)),
        });
    }

    // array: T[N]
    if let Some((inner, size_part)) = token.split_once('[') {
        if let Some(size_str) = size_part.strip_suffix(']') {
            if !size_str.is_empty() {
                let n: usize = size_str
                    .parse()
                    .map_err(|_| anyhow!("Invalid array size in type '{}'", token))?;
                let inner_ty = parse_type_token(current_pkg, default_ns, inner.trim())?;
                return Ok(FieldType::Array(Box::new(inner_ty), n));
            }
        }
    }

    // sequence: sequence<T> or sequence<T, N>
    if let Some(rest) = token.strip_prefix("sequence<") {
        let rest = rest
            .strip_suffix('>')
            .ok_or_else(|| anyhow!("Invalid sequence type '{}'", token))?;
        let parts: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();
        if parts.is_empty() {
            return Err(anyhow!("Invalid sequence type '{}'", token));
        }
        let elem = parse_type_token(current_pkg, default_ns, parts[0])?;
        if parts.len() == 1 {
            return Ok(FieldType::UnboundedSequence(Box::new(elem)));
        }
        if parts.len() == 2 {
            let n: usize = parts[1]
                .parse()
                .map_err(|_| anyhow!("Invalid sequence bound in type '{}'", token))?;
            return Ok(FieldType::BoundedSequence(Box::new(elem), n));
        }
        return Err(anyhow!("Invalid sequence type '{}'", token));
    }

    if let Some(bt) = parse_base_type(token) {
        return Ok(FieldType::Base(bt));
    }

    Ok(FieldType::Named(normalize_named_type(
        current_pkg,
        token,
        default_ns,
    )))
}

fn parse_field_line(current_pkg: &str, default_ns: &str, line: &str) -> Result<Option<Field>> {
    let line = strip_comment(line).trim();
    if line.is_empty() {
        return Ok(None);
    }
    // constants: `type NAME=value` -> ignore for prototype
    if line.contains('=') {
        return Ok(None);
    }

    let mut parts = line.split_whitespace();
    let ty_token = parts
        .next()
        .ok_or_else(|| anyhow!("Invalid field line '{}'", line))?;
    let name = parts
        .next()
        .ok_or_else(|| anyhow!("Invalid field line '{}'", line))?;
    let ty = parse_type_token(current_pkg, default_ns, ty_token)?;
    Ok(Some(Field {
        name: name.to_string(),
        ty,
    }))
}

pub fn parse_msg(current_pkg: &str, text: &str) -> Result<MessageSpec> {
    let mut fields = Vec::new();
    for line in text.lines() {
        if let Some(f) = parse_field_line(current_pkg, "msg", line)? {
            fields.push(f);
        }
    }
    Ok(MessageSpec { fields })
}

pub fn parse_srv(current_pkg: &str, text: &str) -> Result<ServiceSpec> {
    let mut req_lines = Vec::new();
    let mut resp_lines = Vec::new();
    let mut in_resp = false;
    for line in text.lines() {
        if strip_comment(line).trim() == "---" {
            in_resp = true;
            continue;
        }
        if in_resp {
            resp_lines.push(line);
        } else {
            req_lines.push(line);
        }
    }
    Ok(ServiceSpec {
        request: parse_msg(current_pkg, &req_lines.join("\n"))?,
        response: parse_msg(current_pkg, &resp_lines.join("\n"))?,
    })
}

pub fn parse_action(current_pkg: &str, text: &str) -> Result<ActionSpec> {
    // action file has 2 separators:
    // goal
    // ---
    // result
    // ---
    // feedback
    let mut sections: Vec<Vec<&str>> = vec![Vec::new()];
    for line in text.lines() {
        if strip_comment(line).trim() == "---" {
            sections.push(Vec::new());
            continue;
        }
        if let Some(section) = sections.last_mut() {
            section.push(line);
        }
    }
    while sections.len() < 3 {
        sections.push(Vec::new());
    }
    Ok(ActionSpec {
        goal: parse_msg(current_pkg, &sections[0].join("\n"))?,
        result: parse_msg(current_pkg, &sections[1].join("\n"))?,
        feedback: parse_msg(current_pkg, &sections[2].join("\n"))?,
    })
}

pub fn parse_interface(type_name: &str, current_pkg: &str, text: &str) -> Result<InterfaceSpec> {
    // type_name is `pkg/msg/Type`, `pkg/srv/Type`, `pkg/action/Type`
    let parts: Vec<&str> = type_name.split('/').collect();
    if parts.len() != 3 {
        return Err(anyhow!("Invalid type name '{}'", type_name));
    }
    match parts[1] {
        "msg" => Ok(InterfaceSpec::Msg(parse_msg(current_pkg, text)?)),
        "srv" => Ok(InterfaceSpec::Srv(parse_srv(current_pkg, text)?)),
        "action" => Ok(InterfaceSpec::Action(parse_action(current_pkg, text)?)),
        _ => Err(anyhow!("Invalid interface kind '{}'", parts[1])),
    }
}

pub fn default_yaml_for_message(
    root_pkg: &str,
    msg: &MessageSpec,
    resolver: &dyn Fn(&str) -> Result<InterfaceSpec>,
) -> Result<serde_yaml::Value> {
    let mut map = BTreeMap::<String, serde_yaml::Value>::new();
    for f in &msg.fields {
        map.insert(
            f.name.clone(),
            default_yaml_for_field(root_pkg, &f.ty, resolver)?,
        );
    }
    Ok(serde_yaml::to_value(map)?)
}

fn scalar_default(bt: &BaseType) -> serde_yaml::Value {
    match bt {
        BaseType::Bool => serde_yaml::Value::Bool(false),
        BaseType::Int8
        | BaseType::UInt8
        | BaseType::Int16
        | BaseType::UInt16
        | BaseType::Int32
        | BaseType::UInt32
        | BaseType::Int64
        | BaseType::UInt64
        | BaseType::Char
        | BaseType::WChar
        | BaseType::Byte => serde_yaml::Value::Number(0.into()),
        BaseType::Float32 | BaseType::Float64 => serde_yaml::Value::Number(0.into()),
        BaseType::String | BaseType::WString => serde_yaml::Value::String(String::new()),
    }
}

fn default_yaml_for_field(
    root_pkg: &str,
    ty: &FieldType,
    resolver: &dyn Fn(&str) -> Result<InterfaceSpec>,
) -> Result<serde_yaml::Value> {
    Ok(match ty {
        FieldType::Base(b) => scalar_default(b),
        FieldType::BoundedString(_n) => serde_yaml::Value::String(String::new()),
        FieldType::BoundedWString(_n) => serde_yaml::Value::String(String::new()),
        FieldType::Array(inner, n) => {
            let v = default_yaml_for_field(root_pkg, inner, resolver)?;
            serde_yaml::Value::Sequence(std::iter::repeat(v).take(*n).collect())
        }
        FieldType::BoundedSequence(inner, _) | FieldType::UnboundedSequence(inner) => {
            // ros2 interface proto uses an empty list for sequences.
            // Keep it empty regardless of element type.
            let _ = inner;
            serde_yaml::Value::Sequence(Vec::new())
        }
        FieldType::Named(name) => {
            let spec = resolver(name)?;
            match spec {
                InterfaceSpec::Msg(m) => default_yaml_for_message(root_pkg, &m, resolver)?,
                InterfaceSpec::Srv(s) => {
                    // If a message references a service (shouldn't), default to request.
                    default_yaml_for_message(root_pkg, &s.request, resolver)?
                }
                InterfaceSpec::Action(a) => {
                    // If a message references an action (shouldn't), default to goal.
                    default_yaml_for_message(root_pkg, &a.goal, resolver)?
                }
            }
        }
    })
}
