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
	"strings"
)

func kubeActionPodsDelete(namespace string, appName string) {
	var (
		args          = app.Args
		command       = "kube action"
		remains       = args.GetRemains(command, "--help")
		_, _, podName = args.GetOptVal(remains, "delete")
	)

	if len(podName) <= len(appName) || podName[:len(appName)] != appName {
		podName = appName + "-" + podName
	}

	out, err := util.Terminal("", "kubectl delete pods %v -n %v", podName, namespace)

	if err != nil {
		fmt.Println(*err)
		os.Exit(1)
	}

	out = strings.TrimSpace(out)
	if out != "" {
		fmt.Printf("%v\n\n", out)
	}
}
