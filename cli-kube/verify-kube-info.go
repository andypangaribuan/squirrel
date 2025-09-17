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
	"squirrel/app"
	"squirrel/util"
	"strings"
)

func verifyKubeInfo() (namespace string, appNames []string) {
	var (
		isError bool
		args    = app.Args
		command = "kube info"
		remains = args.GetRemains(command, "--help", "--watch")
	)

	if remains == "" && args.IsOptHelp {
		return
	}

	opt, optVal, val := args.GetOptVal(remains, "-n", "--namespace")
	namespace = val
	appNames = make([]string, 0)

	isError = opt == "" || namespace == ""
	if !isError {
		remains = strings.TrimSpace(strings.Replace(remains, optVal, "", 1))
		ls := strings.Split(remains, singleSpace)

		for _, v := range ls {
			if len(v) > 0 && v[0:1] == "-" {
				isError = true
				break
			}

			appNames = append(appNames, v)
		}
	}

	if isError {
		util.UnknownCommand(remains, "run 'sq kube info --help' for more information")
	}

	return namespace, appNames
}
