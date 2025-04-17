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
)

func kubeActionPodsWatch(namespace string, appName string) {
	err := util.InteractiveTerminal("", fmt.Sprintf(
		`watch -t -n 1 "%v kube info %v -n %v"`,
		app.SqCli, appName, namespace))

	if err != nil {
		fmt.Printf("%+v\n", err)
		os.Exit(1)
	}
}
