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
	"squirrel/util"
	"strings"
)

func kubeActionPodsRollout(namespace string, appName string) {
	out, err := util.Terminal("", "kubectl rollout restart deploy %v -n %v", appName, namespace)

	if err != nil {
		fmt.Println(*err)
		os.Exit(1)
	}

	out = strings.TrimSpace(out)
	if out != "" {
		fmt.Printf("%v\n\n", out)
	}
}
