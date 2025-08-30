# OpenAPI Code Generation Guide

This guide explains how to generate or regenerate the Lighter Rust SDK using OpenAPI Generator.

## Prerequisites

### Install OpenAPI Generator

#### Option 1: Using Homebrew (macOS/Linux)
```bash
brew install openapi-generator
```

#### Option 2: Using npm
```bash
npm install -g @openapitools/openapi-generator-cli
```

#### Option 3: Using Docker
```bash
docker pull openapitools/openapi-generator-cli
```

#### Option 4: Download JAR directly
```bash
wget https://repo1.maven.org/maven2/org/openapitools/openapi-generator-cli/7.0.1/openapi-generator-cli-7.0.1.jar -O openapi-generator-cli.jar
```

## Generating the SDK

### Using the Configuration File

The repository includes a pre-configured generator setup in `.openapi-generator/generator-config.yaml`:

```bash
# Navigate to the .openapi-generator directory
cd .openapi-generator

# Generate using the configuration
openapi-generator-cli generate \
  -c generator-config.yaml
```

### Manual Generation

To generate the SDK manually with custom options:

```bash
openapi-generator-cli generate \
  -i openapi.yaml \
  -g rust \
  -o . \
  --package-name lighter-rust \
  --library reqwest \
  --additional-properties=supportAsync=true,supportMultipleResponses=true
```

### Using Docker

```bash
docker run --rm \
  -v ${PWD}:/local openapitools/openapi-generator-cli generate \
  -i /local/openapi.yaml \
  -g rust \
  -o /local \
  -c /local/.openapi-generator/generator-config.yaml
```

## Configuration Options

The generator configuration (`generator-config.yaml`) includes:

```yaml
generatorName: rust
outputDir: ../
inputSpec: https://api.lighter.xyz/openapi.json  # or local file
additionalProperties:
  packageName: lighter-rust
  packageVersion: 0.1.0
  library: reqwest              # HTTP client library
  supportAsync: true            # Enable async/await
  supportMultipleResponses: true
  preferUnsignedInt: false
  bestFitInt: true
  hideGenerationTimestamp: true
  useSingleRequestParameter: false
  avoidBoxedModels: true
```

## Selective Generation

To generate only specific components:

### Generate Only Models
```bash
openapi-generator-cli generate \
  -i openapi.yaml \
  -g rust \
  -o . \
  --global-property models="Account,Order,Balance"
```

### Generate Only APIs
```bash
openapi-generator-cli generate \
  -i openapi.yaml \
  -g rust \
  -o . \
  --global-property apis="AccountApi,OrderApi"
```

### Generate Specific Files
```bash
openapi-generator-cli generate \
  -i openapi.yaml \
  -g rust \
  -o . \
  --global-property supportingFiles="Cargo.toml,lib.rs"
```

## Preserving Custom Code

The `.openapi-generator-ignore` file prevents overwriting of custom implementations:

```
# Custom implementations - do not overwrite
src/signers/**
src/nonce.rs
src/lib.rs
examples/**
tests/**
Cargo.toml
README.md
```

## Updating the OpenAPI Specification

### From Remote API
```bash
# Download latest spec
curl https://api.lighter.xyz/openapi.json -o openapi.json

# Convert to YAML (optional)
yq eval -P openapi.json > openapi.yaml

# Regenerate SDK
openapi-generator-cli generate -c .openapi-generator/generator-config.yaml
```

### Validating the Spec
```bash
# Validate OpenAPI specification
openapi-generator-cli validate -i openapi.yaml
```

## Post-Generation Steps

After generating the SDK:

1. **Review Changes**
   ```bash
   git diff
   ```

2. **Fix Compilation Issues**
   ```bash
   cargo check
   cargo fmt
   cargo clippy
   ```

3. **Run Tests**
   ```bash
   cargo test
   ```

4. **Update Documentation**
   ```bash
   cargo doc --open
   ```

## Customizing Templates

For advanced customization, you can modify the generator templates:

1. **Export Templates**
   ```bash
   openapi-generator-cli author template -g rust -o templates
   ```

2. **Modify Templates**
   Edit files in `templates/` directory

3. **Generate with Custom Templates**
   ```bash
   openapi-generator-cli generate \
     -i openapi.yaml \
     -g rust \
     -o . \
     -t templates
   ```

## Common Issues and Solutions

### Issue: Duplicate Model Definitions
**Solution**: Use `--skip-validate-spec` flag or clean up the OpenAPI spec

### Issue: Invalid Rust Code Generated
**Solution**: Update to latest OpenAPI Generator version or use custom templates

### Issue: Missing Async Support
**Solution**: Ensure `supportAsync=true` in additional properties

### Issue: Conflicting Dependencies
**Solution**: Review and update Cargo.toml dependencies manually

## Automation with Make

Create a `Makefile` for easy regeneration:

```makefile
.PHONY: generate validate clean

generate:
	@echo "Generating Rust SDK from OpenAPI spec..."
	@cd .openapi-generator && openapi-generator-cli generate -c generator-config.yaml
	@cargo fmt
	@cargo check

validate:
	@echo "Validating OpenAPI specification..."
	@openapi-generator-cli validate -i openapi.yaml

clean:
	@echo "Cleaning generated files..."
	@rm -rf src/apis src/models docs/apis docs/models

update-spec:
	@echo "Downloading latest OpenAPI spec..."
	@curl https://api.lighter.xyz/openapi.json -o openapi.json
	@yq eval -P openapi.json > openapi.yaml

regenerate: clean update-spec generate
	@echo "SDK regenerated successfully!"
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Generate SDK

on:
  schedule:
    - cron: '0 0 * * 0' # Weekly
  workflow_dispatch:

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install OpenAPI Generator
        run: npm install -g @openapitools/openapi-generator-cli
      
      - name: Generate SDK
        run: |
          cd .openapi-generator
          openapi-generator-cli generate -c generator-config.yaml
      
      - name: Check for changes
        run: |
          if [ -n "$(git status --porcelain)" ]; then
            git config user.name "GitHub Actions"
            git config user.email "actions@github.com"
            git add .
            git commit -m "chore: regenerate SDK from OpenAPI spec"
            git push
          fi
```

## Version Management

Track OpenAPI Generator version in `.openapi-generator/VERSION`:
```
7.0.1
```

Update to a new version:
```bash
# Update version file
echo "7.1.0" > .openapi-generator/VERSION

# Install specific version
npm install -g @openapitools/openapi-generator-cli@7.1.0

# Regenerate
openapi-generator-cli generate -c .openapi-generator/generator-config.yaml
```

## Additional Resources

- [OpenAPI Generator Documentation](https://openapi-generator.tech/docs/generators/rust/)
- [Rust Generator Config Options](https://openapi-generator.tech/docs/generators/rust/#config-options)
- [OpenAPI Specification](https://spec.openapis.org/oas/latest.html)
- [Lighter API Documentation](https://apibetadocs.lighter.xyz/docs)