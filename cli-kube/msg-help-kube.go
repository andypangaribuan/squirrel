/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package clikube

import (
	"squirrel/util"

	"github.com/wissance/stringFormatter"
)

// sq kube --msgHelpKube
var msgHelpKube = stringFormatter.FormatComplex(`
info : Execute kubectl cli
usage: sq kube [commands] [options]

{commands}
  info     Show application information
  action   Comprehensive kubectl execution
`, map[string]any{
	"commands": util.ColorBoldGreen("commands:"),
})
