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

func execKubeActionDiff(optValue string, ymlTemplates []string) {
	lines := getYmlLines(optValue, ymlTemplates)
	script := fmt.Sprintf(`cat <<'EOF' | kubectl diff -f -
%v
EOF`, lines)

	out, err := util.Terminal("", script)
	if err != nil {
		fmt.Println(*err)
		os.Exit(1)
	}

	out = strings.TrimSpace(out)
	var (
		ls    = strings.Split(out, "\n")
		count = len(ls)
		rm    = []string{
			"diff -u -N /var/folders/rn/",
			"--- /var/folders/rn/",
			"+++ /var/folders/rn/",
			"@@ ",
		}
	)

	for i := 0; i < count; i++ {
		line := ls[i]
		isContinue := false

		for _, r := range rm {
			if len(line) > len(r) && line[:len(r)] == r {
				ls = append(ls[:i], ls[i+1:]...)
				isContinue = true
				i--
				count--
				break
			}
		}

		if isContinue {
			continue
		}
	}

	if len(ls) > 0 && ls[0] != "" {
		if len(ls) == 1 {
			fmt.Println(ls[0])
		} else {
			fmt.Println(strings.Join(ls, "\n") + "\n")
		}
	}
}
