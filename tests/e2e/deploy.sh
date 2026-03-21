#!/usr/bin/env bash
# Deploy a SAM project to AWS
#
# Usage:
#   bash tests/e2e/deploy.sh <project-dir>
#   bash tests/e2e/deploy.sh --help
#
# Environment:
#   AWS_PROFILE   AWS profile (default: default)
#   AWS_REGION    AWS region (default: us-east-1)
#   SAM_S3_BUCKET Override S3 bucket for artifacts (optional)
#
# Output:
#   Writes deploy-output.toml in project dir with stack_name, api_url, region

set -euo pipefail

if [[ "${1:-}" == "--help" || -z "${1:-}" ]]; then
  echo "Usage: bash deploy.sh <project-dir>"
  echo ""
  echo "Deploys a SAM project to AWS. Skips if no credentials or sam CLI."
  echo ""
  echo "Environment variables:"
  echo "  AWS_PROFILE    AWS profile (default: default)"
  echo "  AWS_REGION     AWS region (default: us-east-1)"
  echo "  SAM_S3_BUCKET  Override S3 bucket (optional)"
  exit 0
fi

PROJECT_DIR="$1"
REGION="${AWS_REGION:-us-east-1}"
DEPLOY_TIMEOUT=600  # 10 minutes

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

# Check prerequisites
if ! command -v sam >/dev/null 2>&1; then
  echo -e "${YELLOW}sam CLI not found — skipping deployment${NC}"
  exit 0
fi

if ! aws sts get-caller-identity >/dev/null 2>&1; then
  echo -e "${YELLOW}No AWS credentials — skipping deployment${NC}"
  exit 0
fi

if [[ ! -f "$PROJECT_DIR/template.yaml" ]]; then
  echo -e "${RED}No template.yaml in $PROJECT_DIR${NC}"
  exit 1
fi

STACK_NAME="konductor-$(basename "$PROJECT_DIR")-$(date +%s)"
OUTPUT_FILE="$PROJECT_DIR/deploy-output.toml"

echo "Deploying: $PROJECT_DIR"
echo "Stack: $STACK_NAME"
echo "Region: $REGION"

# Step 1: Validate
echo -e "\n${GREEN}▶ sam validate${NC}"
(cd "$PROJECT_DIR" && sam validate --region "$REGION") || { echo -e "${RED}Validation failed${NC}"; exit 1; }

# Step 2: Build
echo -e "\n${GREEN}▶ sam build${NC}"
(cd "$PROJECT_DIR" && sam build) || { echo -e "${RED}Build failed${NC}"; exit 1; }

# Step 3: Deploy
echo -e "\n${GREEN}▶ sam deploy${NC}"
DEPLOY_ARGS=(
  --stack-name "$STACK_NAME"
  --region "$REGION"
  --capabilities CAPABILITY_IAM
  --no-confirm-changeset
  --no-fail-on-empty-changeset
)

if [[ -n "${SAM_S3_BUCKET:-}" ]]; then
  DEPLOY_ARGS+=(--s3-bucket "$SAM_S3_BUCKET")
else
  DEPLOY_ARGS+=(--resolve-s3)
fi

if ! timeout "$DEPLOY_TIMEOUT" bash -c "cd '$PROJECT_DIR' && sam deploy ${DEPLOY_ARGS[*]}" 2>&1; then
  echo -e "${RED}Deploy failed or timed out${NC}"
  # Write partial output so teardown knows the stack name
  cat > "$OUTPUT_FILE" << EOF
stack_name = "$STACK_NAME"
region = "$REGION"
api_url = ""
deploy_success = false
EOF
  exit 1
fi

# Step 4: Extract API URL from stack outputs
API_URL=""
OUTPUTS=$(aws cloudformation describe-stacks \
  --stack-name "$STACK_NAME" \
  --region "$REGION" \
  --query "Stacks[0].Outputs" \
  --output json 2>/dev/null || echo "[]")

API_URL=$(echo "$OUTPUTS" | python3 -c "
import json, sys
outputs = json.load(sys.stdin)
for o in (outputs or []):
    if 'api' in o.get('OutputKey','').lower() or 'url' in o.get('OutputKey','').lower() or 'endpoint' in o.get('OutputKey','').lower():
        print(o['OutputValue'])
        break
" 2>/dev/null || echo "")

echo -e "\n${GREEN}Deploy complete${NC}"
echo "API URL: ${API_URL:-<not found>}"

cat > "$OUTPUT_FILE" << EOF
stack_name = "$STACK_NAME"
region = "$REGION"
api_url = "$API_URL"
deploy_success = true
EOF

echo "Output written to: $OUTPUT_FILE"
