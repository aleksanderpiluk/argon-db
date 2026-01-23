#!/bin/bash
set -e

PROTO_ROOT=../libargonconnector-grpc/proto
OUT_DIR=.

python -m grpc_tools.protoc \
  -I"${PROTO_ROOT}" \
  -I"$(python - <<'EOF'
import grpc_tools
print(grpc_tools.__path__[0] + "/_proto")
EOF
)" \
  --python_out="${OUT_DIR}" \
  --grpc_python_out="${OUT_DIR}" \
  ${PROTO_ROOT}/*.proto
