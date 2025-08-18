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
	"fmt"
	"os"
	"squirrel/app"
	"squirrel/util"
	"strconv"
	"strings"

	"github.com/andypangaribuan/gmod/fm"
)

func dockerPs(doPrint ...bool) string {
	args := app.Args

	command := "docker ps"
	command = args.AddRemains(command, "--watch")

	out, err := util.Terminal("", command)
	if err != nil {
		fmt.Println(*err)
		os.Exit(1)
	}

	keys, vals := util.MapKV(out, "CONTAINER ID", "IMAGE", "COMMAND", "CREATED", "STATUS", "PORTS", "NAMES")

	var (
		items     = make([][]any, 0)
		idxName   = keys["NAMES"]
		idxStatus = keys["STATUS"]
		idxPort   = keys["PORTS"]
		idxImage  = keys["IMAGE"]
	)

	// set header
	items = append(items, []any{"", "NAMES", "STATUS", "PORTS", "IMAGE"})

	// set items
	for i, ls := range vals {
		activePorts := make([]string, 0)

		port := util.VTrim(ls, idxPort)
		if port != "" {
			ls := strings.Split(port, ",")
			for _, v := range ls {
				v = strings.TrimSpace(v)
				if strings.Contains(v, "->") {
					ipPort := strings.Split(v, "->")[0]
					if strings.Contains(ipPort, ":") {
						ls := strings.Split(ipPort, ":")
						last := ls[len(ls)-1]
						_, err := strconv.Atoi(last)
						if err == nil && !fm.IfHaveIn(last, activePorts...) {
							activePorts = append(activePorts, last)
						}
					}
				}
			}
		}

		items = append(items, []any{
			i + 1,
			util.VTrim(ls, idxName),
			util.VTrim(ls, idxStatus),
			strings.Join(activePorts, ", "),
			util.VTrim(ls, idxImage),
		})
	}

	output := util.Build(items)
	printOutput := false

	if len(doPrint) > 0 {
		printOutput = doPrint[0]
	}

	if printOutput {
		fmt.Println(output)
	}

	return output
}
