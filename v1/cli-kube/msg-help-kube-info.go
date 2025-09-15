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

// sq kube info --help
var msgHelpKubeInfo = stringFormatter.FormatComplex(`
info : Show application information
usage: sq kube info {app-name} [options]

{commands}
  -n, --namespace {string}   Application namespace
      --watch                Stream every second
`, map[string]any{
	"commands": util.ColorBoldGreen("options:"),
})
