/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package kube

import (
	"squirrel/arg"
	"squirrel/util"

	"github.com/wissance/stringFormatter"
)

func CLI() {
	helpMessage := stringFormatter.FormatComplex(`
info : execute kubectl cli
usage: sq kube

{commands}
  pods     show pods information
  action   comprehensive kubectl execution
`, map[string]any{
		"commands": util.ColorBoldGreen("commands:"),
	})

	arg.Watch("sq kube", helpMessage, helpMessage).
		Add("pods", "", cliKubePods).
		Add("action", "", cliKubeAction).
		Exec()
}
