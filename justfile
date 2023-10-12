build:
    cargo build

book:
    mdbook build book --dest-dir ../docs

do type:
    #!/usr/bin/env bash
    cargo release version {{type}} --execute
    version=v$(cat Cargo.toml | grep version | head -1 | choose 2 | tr -d ,\")
    git cliff --tag $version > CHANGELOG.md
    changelog=$(git cliff --tag $version --strip all)
    git add -A && git commit -m "chore(release): prepare for $version"
    git tag -a $version -m "$version" -m "$changelog"
    git push --follow-tags --force --set-upstream origin develop
    gh release create $version --notes "$changelog"


default:
    echo 'Hello, world!'
