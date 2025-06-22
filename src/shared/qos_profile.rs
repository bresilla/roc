use rclrs::*;

/// QoS Profile information
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct QosProfile {
    pub history: QosHistoryPolicy,
    pub depth: usize,
    pub reliability: QosReliabilityPolicy,
    pub durability: QosDurabilityPolicy,
    pub deadline_sec: u64,
    pub deadline_nsec: u64,
    pub lifespan_sec: u64,
    pub lifespan_nsec: u64,
    pub liveliness: QosLivelinessPolicy,
    pub liveliness_lease_duration_sec: u64,
    pub liveliness_lease_duration_nsec: u64,
    pub avoid_ros_namespace_conventions: bool,
}

/// QoS History Policy
#[derive(Debug, Clone)]
pub enum QosHistoryPolicy {
    SystemDefault,
    KeepLast,
    KeepAll,
    Unknown,
}

/// QoS Reliability Policy
#[derive(Debug, Clone)]
pub enum QosReliabilityPolicy {
    SystemDefault,
    Reliable,
    BestEffort,
    Unknown,
    BestAvailable,
}

/// QoS Durability Policy
#[derive(Debug, Clone)]
pub enum QosDurabilityPolicy {
    SystemDefault,
    TransientLocal,
    Volatile,
    Unknown,
    BestAvailable,
}

/// QoS Liveliness Policy
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum QosLivelinessPolicy {
    SystemDefault,
    Automatic,
    ManualByNode,
    ManualByTopic,
    Unknown,
    BestAvailable,
}

impl QosHistoryPolicy {
    #[allow(non_upper_case_globals)]
    pub fn from_rmw(history: rmw_qos_history_policy_e) -> Self {
        match history {
            rmw_qos_history_policy_e_RMW_QOS_POLICY_HISTORY_SYSTEM_DEFAULT => QosHistoryPolicy::SystemDefault,
            rmw_qos_history_policy_e_RMW_QOS_POLICY_HISTORY_KEEP_LAST => QosHistoryPolicy::KeepLast,
            rmw_qos_history_policy_e_RMW_QOS_POLICY_HISTORY_KEEP_ALL => QosHistoryPolicy::KeepAll,
            _ => QosHistoryPolicy::Unknown,
        }
    }
    
    pub fn to_string(&self) -> &'static str {
        match self {
            QosHistoryPolicy::SystemDefault => "SYSTEM_DEFAULT",
            QosHistoryPolicy::KeepLast => "KEEP_LAST",
            QosHistoryPolicy::KeepAll => "KEEP_ALL",
            QosHistoryPolicy::Unknown => "UNKNOWN",
        }
    }
}

impl QosReliabilityPolicy {
    #[allow(non_upper_case_globals)]
    pub fn from_rmw(reliability: rmw_qos_reliability_policy_e) -> Self {
        match reliability {
            rmw_qos_reliability_policy_e_RMW_QOS_POLICY_RELIABILITY_SYSTEM_DEFAULT => QosReliabilityPolicy::SystemDefault,
            rmw_qos_reliability_policy_e_RMW_QOS_POLICY_RELIABILITY_RELIABLE => QosReliabilityPolicy::Reliable,
            rmw_qos_reliability_policy_e_RMW_QOS_POLICY_RELIABILITY_BEST_EFFORT => QosReliabilityPolicy::BestEffort,
            rmw_qos_reliability_policy_e_RMW_QOS_POLICY_RELIABILITY_BEST_AVAILABLE => QosReliabilityPolicy::BestAvailable,
            _ => QosReliabilityPolicy::Unknown,
        }
    }
    
    pub fn to_string(&self) -> &'static str {
        match self {
            QosReliabilityPolicy::SystemDefault => "SYSTEM_DEFAULT",
            QosReliabilityPolicy::Reliable => "RELIABLE",
            QosReliabilityPolicy::BestEffort => "BEST_EFFORT",
            QosReliabilityPolicy::Unknown => "UNKNOWN",
            QosReliabilityPolicy::BestAvailable => "BEST_AVAILABLE",
        }
    }
}

impl QosDurabilityPolicy {
    #[allow(non_upper_case_globals)]
    pub fn from_rmw(durability: rmw_qos_durability_policy_e) -> Self {
        match durability {
            rmw_qos_durability_policy_e_RMW_QOS_POLICY_DURABILITY_SYSTEM_DEFAULT => QosDurabilityPolicy::SystemDefault,
            rmw_qos_durability_policy_e_RMW_QOS_POLICY_DURABILITY_TRANSIENT_LOCAL => QosDurabilityPolicy::TransientLocal,
            rmw_qos_durability_policy_e_RMW_QOS_POLICY_DURABILITY_VOLATILE => QosDurabilityPolicy::Volatile,
            rmw_qos_durability_policy_e_RMW_QOS_POLICY_DURABILITY_BEST_AVAILABLE => QosDurabilityPolicy::BestAvailable,
            _ => QosDurabilityPolicy::Unknown,
        }
    }
    
    pub fn to_string(&self) -> &'static str {
        match self {
            QosDurabilityPolicy::SystemDefault => "SYSTEM_DEFAULT",
            QosDurabilityPolicy::TransientLocal => "TRANSIENT_LOCAL",
            QosDurabilityPolicy::Volatile => "VOLATILE",
            QosDurabilityPolicy::Unknown => "UNKNOWN",
            QosDurabilityPolicy::BestAvailable => "BEST_AVAILABLE",
        }
    }
}

impl QosLivelinessPolicy {
    #[allow(non_upper_case_globals)]
    pub fn from_rmw(liveliness: rmw_qos_liveliness_policy_e) -> Self {
        match liveliness {
            rmw_qos_liveliness_policy_e_RMW_QOS_POLICY_LIVELINESS_SYSTEM_DEFAULT => QosLivelinessPolicy::SystemDefault,
            rmw_qos_liveliness_policy_e_RMW_QOS_POLICY_LIVELINESS_AUTOMATIC => QosLivelinessPolicy::Automatic,
            rmw_qos_liveliness_policy_e_RMW_QOS_POLICY_LIVELINESS_MANUAL_BY_TOPIC => QosLivelinessPolicy::ManualByTopic,
            rmw_qos_liveliness_policy_e_RMW_QOS_POLICY_LIVELINESS_BEST_AVAILABLE => QosLivelinessPolicy::BestAvailable,
            _ => QosLivelinessPolicy::Unknown,
        }
    }
    
    pub fn to_string(&self) -> &'static str {
        match self {
            QosLivelinessPolicy::SystemDefault => "SYSTEM_DEFAULT",
            QosLivelinessPolicy::Automatic => "AUTOMATIC",
            QosLivelinessPolicy::ManualByNode => "MANUAL_BY_NODE",
            QosLivelinessPolicy::ManualByTopic => "MANUAL_BY_TOPIC",
            QosLivelinessPolicy::Unknown => "UNKNOWN",
            QosLivelinessPolicy::BestAvailable => "BEST_AVAILABLE",
        }
    }
}

impl QosProfile {
    pub fn from_rmw(qos: &rmw_qos_profile_t) -> Self {
        QosProfile {
            history: QosHistoryPolicy::from_rmw(qos.history),
            depth: qos.depth,
            reliability: QosReliabilityPolicy::from_rmw(qos.reliability),
            durability: QosDurabilityPolicy::from_rmw(qos.durability),
            deadline_sec: qos.deadline.sec,
            deadline_nsec: qos.deadline.nsec,
            lifespan_sec: qos.lifespan.sec,
            lifespan_nsec: qos.lifespan.nsec,
            liveliness: QosLivelinessPolicy::from_rmw(qos.liveliness),
            liveliness_lease_duration_sec: qos.liveliness_lease_duration.sec,
            liveliness_lease_duration_nsec: qos.liveliness_lease_duration.nsec,
            avoid_ros_namespace_conventions: qos.avoid_ros_namespace_conventions,
        }
    }
    
    pub fn format_duration(&self, sec: u64, nsec: u64) -> String {
        if sec == 0x7FFFFFFFFFFFFFFF && nsec == 0x7FFFFFFFFFFFFFFF {
            "infinite".to_string()
        } else if sec == 0 && nsec == 0 {
            "0.000000000".to_string()
        } else {
            format!("{}.{:09}", sec, nsec)
        }
    }
}