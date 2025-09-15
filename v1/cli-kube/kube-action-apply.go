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

	"github.com/andypangaribuan/gmod/fm"
)

func kubeActionApply(lsYml []string, lsYmlTemplate []string) {
	var (
		args         = app.Args
		command      = "kube action"
		remains      = args.GetRemains(command, "--help")
		_, _, optVal = args.GetOptVal(remains, "apply")
	)

	if !fm.IfHaveIn(optVal, lsYml...) {
		fmt.Printf("%v\n\n", util.ColorBoldRed("yml not available"))
		os.Exit(1)
	}

	lines := getYmlLines(lsYmlTemplate, optVal)
	script := fmt.Sprintf(`cat <<'EOF' | kubectl apply -f -
%v
EOF`, lines)

	out, err := util.Terminal("", script)
	if err != nil {
		fmt.Println(*err)
		os.Exit(1)
	}

	fmt.Println(fm.Ternary(len(strings.Split(strings.TrimSpace(out), "\n")) == 1, strings.TrimSpace(out), out))
	os.Exit(0)
}
