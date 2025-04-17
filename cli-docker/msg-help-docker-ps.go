/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package clidocker

import (
	"squirrel/util"

	"github.com/wissance/stringFormatter"
)

// sq docker ps --help
var msgHelpDockerPs = stringFormatter.FormatComplex(`
info : List containers
usage: sq docker ps [options]

{options}
  -a, --all     Show all containers (default show just running)
      --watch   Stream every second
`, map[string]any{
	"options": util.ColorBoldGreen("options:"),
})
