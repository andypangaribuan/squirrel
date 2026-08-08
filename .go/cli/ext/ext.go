/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package ext

import (
	"squirrel/arg"
	"squirrel/util"

	"github.com/wissance/stringFormatter"
)

func CLI() {
	helpMessage := stringFormatter.FormatComplex(`
info : execute ext cli
usage: sq ext

{commands}
  rgo   robot go
`, map[string]any{
		"commands": util.ColorBoldGreen("commands:"),
	})

	arg.Watch("sq ext", helpMessage, helpMessage).
		Add("rgo", "", cliRgo).
		Exec()
}
