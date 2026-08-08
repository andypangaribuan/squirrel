/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package kube

import (
	"fmt"
	"os"
	"squirrel/util"
	"strings"
)

func execKubeActionApply(optValue string, ymlTemplates []string) {
	lines := getYmlLines(optValue, ymlTemplates)
	script := fmt.Sprintf(`cat <<'EOF' | kubectl apply -f -
%v
EOF`, lines)

	out, err := util.Terminal("", script)
	if err != nil {
		fmt.Println(*err)
		os.Exit(1)
	}

	out = strings.TrimSpace(out)
	if len(strings.Split(out, "\n")) == 1 {
		if out != "" {
			fmt.Println(out)
		}
	} else {
		fmt.Println(out + "\n")
	}
}
