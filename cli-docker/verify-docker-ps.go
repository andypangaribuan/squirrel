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
	"squirrel/app"
	"squirrel/util"
)

func verifyDockerPs() {
	args := app.Args
	command := "docker ps"
	remains := args.GetRemains(command, "--help", "--watch", "-a")

	if remains != "" {
		util.UnknownCommand(remains, "run 'sq docker ps --help' for more information")
	}
}
