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
	"fmt"
	"os"
	"squirrel/app"
	"squirrel/util"

	"github.com/wissance/stringFormatter"
)

func kubeActionPodsLogs(namespace string, appName string) {
	var (
		args         = app.Args
		since        = "60m"
		command      = "kube action"
		remains      = args.GetRemains(command, "--help")
		_, _, optVal = args.GetOptVal(remains, "logs")
	)

	if optVal != "" {
		since = optVal
	}

	err := util.InteractiveTerminal("",
		stringFormatter.FormatComplex(
			`stern -n {namespace} {app} -c {app} -l app={app} -t --since {since}`,
			map[string]any{
				"namespace": namespace,
				"app":       appName,
				"since":     since,
			}))

	if err != nil {
		fmt.Printf("%+v\n", err)
		os.Exit(1)
	}
}
