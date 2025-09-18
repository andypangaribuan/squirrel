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
	"encoding/base64"
	"fmt"
	"os"
	"squirrel/util"
	"strings"

	"github.com/andypangaribuan/gmod/fm"
	"github.com/andypangaribuan/gmod/gm"
)

func execKubeActionSecret(namespace string, appName string) {
	script := "kubectl get secret " + appName
	script += fm.Ternary(namespace == "", "", " -n "+namespace)
	script += " -o json"

	out, err := util.Terminal("", script)
	if err != nil {
		fmt.Println(*err)
		os.Exit(1)
	}

	if strings.Contains(out, "(NotFound)") {
		util.PrintThenExit(out, true)
	}

	type model struct {
		Data map[string]string `json:"data"`
	}

	var m model

	if err := gm.Json.Decode(out, &m); err != nil {
		util.PrintThenExit(err.Error(), true)
	}

	out = ""
	for key, val := range m.Data {
		out += fm.Ternary(out == "", "", "\n\n\n")
		out += util.ColorBoldRed(key) + "\n"

		decodedBytes, err := base64.StdEncoding.DecodeString(val)
		if err != nil {
			util.PrintThenExit(err.Error(), true)
		}

		lines := strings.Split(string(decodedBytes), "\n")
		for i, line := range lines {
			if len(line) > 0 && line[:1] == "#" {
				lines[i] = util.ColorCyan(line)
				continue
			}

			ls := strings.Split(line, "=")
			if len(ls) == 2 && util.ContainsOnlyAlphanumericAndUnderscore(ls[0]) {
				lines[i] = util.ColorGreen(ls[0]) + util.ColorYellow("=")

				if util.ContainsOnlyNumeric(ls[1]) {
					lines[i] += util.ColorRed(ls[1])
				} else {
					lines[i] += ls[1]
				}
			}
		}

		out += strings.TrimSpace(strings.Join(lines, "\n"))
	}

	util.PrintThenExit(out)
}
