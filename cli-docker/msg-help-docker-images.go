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

// sq docker images --help
var msgHelpDockerImages = stringFormatter.FormatComplex(`
info : List docker image
usage: sq docker images [options]

{options}
  -c, --contains   [optional] Filter by image name
`, map[string]any{
	"options": util.ColorBoldGreen("options:"),
})
