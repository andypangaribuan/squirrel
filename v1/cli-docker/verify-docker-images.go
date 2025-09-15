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
	"strings"
)

func verifyDockerImages() (contains string) {
	args := app.Args
	isError := false

	command := "docker images"
	remains := args.GetRemains(command, "--help", "--watch")
	opt, optVal, ovContains := args.GetOptVal(remains, "-c", "--contains")
	if opt != "" && ovContains == "" {
		isError = true
	}

	if !isError {
		remains = strings.TrimSpace(strings.Replace(remains, optVal, "", 1))
		isError = remains != ""
	}

	if isError {
		util.UnknownCommand(remains, "run 'sq docker images --help' for more information")
	}

	return ovContains
}
