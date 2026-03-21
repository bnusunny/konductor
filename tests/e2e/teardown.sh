#!/usr/bin/env bash
# Teardown a deployed SAM stack
#
# Usage:
#   bash tests/e2e/teardown.sh <project-dir>
#   bash tests/e2e/teardown.sh --help
#
# Reads deploy-output.toml from project dir for stack name and region.

set -euo pipefail

if [[ "${1:-}" == "--help" || -z "${1:-}" ]]; then
  echo "Usage: bash teardown.sh <project-dir>"
  echo ""
  echo "Deletes the SAM stack. Reads deploy-output.toml for stack name."
  echo "Falls back to forced CloudFormation delete on failure."
  exit 0
fi

PROJECT_DIR="$1"
DEPLOY_OUTPUT="$PROJECT_DIR/deploy-output.toml"
DELETE_TIMEOUT=300  # 5 minutes

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

if [[ ! -f "$DEPLOY_OUTPUT" ]]; then
  echo -e "${YELLOW}No deploy-output.toml — nothing to tear down${NC}"
  exit 0
fi

STACK_NAME=$(grep "^stack_name" "$DEPLOY_OUTPUT" | sed 's/.*= *"//' | tr -d '"')
REGION=$(grep "^region" "$DEPLOY_OUTPUT" | sed 's/.*= *"//' | tr -d '"')
REGION="${REGION:-${AWS_REGION:-us-east-1}}"

if [[ -z "$STACK_NAME" ]]; then
  echo -e "${YELLOW}No stack name in deploy-output.toml — nothing to tear down${NC}"
  exit 0
fi

echo "Tearing down: $STACK_NAME (region: $REGION)"

# Try sam delete first
echo -e "${GREEN}▶ sam delete${NC}"
if timeout "$DELETE_TIMEOUT" sam delete \
  --stack-name "$STACK_NAME" \
  --region "$REGION" \
  --no-prompts 2>&1; then
  echo -e "${GREEN}Stack deleted successfully${NC}"
  exit 0
fi

# Fallback: forced CloudFormation delete
echo -e "${YELLOW}sam delete failed — attempting forced CloudFormation delete${NC}"
aws cloudformation delete-stack \
  --stack-name "$STACK_NAME" \
  --region "$REGION" 2>&1 || true

# Wait for deletion
echo "Waiting for stack deletion..."
if timeout "$DELETE_TIMEOUT" aws cloudformation wait stack-delete-complete \
  --stack-name "$STACK_NAME" \
  --region "$REGION" 2>&1; then
  echo -e "${GREEN}Stack deleted (forced)${NC}"
  exit 0
fi

echo -e "${RED}Stack deletion timed out — manual cleanup may be needed: $STACK_NAME${NC}"
exit 1
