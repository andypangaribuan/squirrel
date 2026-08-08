/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package docker

import (
	"squirrel/arg"
	"squirrel/util"

	"github.com/wissance/stringFormatter"
)

func CLI() {
	helpMessage := stringFormatter.FormatComplex(`
info : execute docker cli
usage: sq docker

{commands}
  ps       list containers
  images   list docker image
`, map[string]any{
		"commands": util.ColorBoldGreen("commands:"),
	})

	arg.Watch("sq docker", helpMessage, helpMessage).
		Add("ps", "", cliDockerPs).
		Add("images", "", cliDockerImages).
		Exec()
}
