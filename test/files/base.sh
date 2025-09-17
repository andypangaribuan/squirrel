#!/bin/bash
# Copyright (c) 2025.
# Created by Andy Pangaribuan. All Rights Reserved.
# This product is protected by copyright and distributed under
# licenses restricting copying, distribution and decompilation.

if [ -f .cm.yml ]; then
  export KYML_CM_DATA=$(echo | cat - .cm.yml | sed 's/^./  &/')
fi

if [ -f .secret.yml ]; then
  export KYML_SECRET_DATA=$(echo | cat - .secret.yml | sed 's/^./  &/')
fi
