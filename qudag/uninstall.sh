#!/usr/bin/env bash

# QuDAG CLI Uninstallation Script
# This script removes the QuDAG CLI installation

set -euo pipefail

# Simply call the install script with uninstall flag
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
"${SCRIPT_DIR}/install.sh" --uninstall "$@"