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

// sq docker --help
var msgHelpDocker = stringFormatter.FormatComplex(`
info : Execute docker cli
usage: sq docker [commands] [options]

{commands}
  ps       List containers
  images   List docker image
`, map[string]any{
	"commands": util.ColorBoldGreen("commands:"),
})
