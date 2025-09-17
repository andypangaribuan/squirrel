/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package docker

import (
	"fmt"
	"os"
	"slices"
	"squirrel/arg"
	"squirrel/util"
	"strconv"
	"strings"

	"github.com/andypangaribuan/gmod/fm"
	"github.com/wissance/stringFormatter"
)

func cliDockerPs() {
	moreInfoMessage := "run 'sq docker ps --help' for more information"
	helpMessage := stringFormatter.FormatComplex(`
info : list containers
usage: sq docker ps

{options}
  -a, --all     show all containers (default show just running)
      --port    [+value] filter by port using contains
      --watch   stream every second
`, map[string]any{
		"options": util.ColorBoldGreen("options:"),
	})

	isOptHelp, index := arg.Search("--help")
	arg.Remove(index)

	isOptAll, index := arg.Search("-a", "--all")
	arg.Remove(index)

	isOptWatch, index := arg.Search("--watch")
	arg.Remove(index)

	portContains := arg.GetOptValue(moreInfoMessage, "--port")

	if arg.Count() > 0 {
		util.UnknownCommand(arg.Remains(), moreInfoMessage)
	}

	if isOptHelp {
		util.PrintThenExit(helpMessage)
	}

	cliOpt := ""
	if isOptAll {
		cliOpt = "-a"
	}

	if isOptWatch {
		util.Watch(func() string {
			return execDockerPs(cliOpt, portContains)
		})
	}

	execDockerPs(cliOpt, portContains, true)
}

func execDockerPs(cliOpt string, portContains string, doPrint ...bool) string {
	command := "docker ps"
	if cliOpt != "" {
		command += " " + cliOpt
	}

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

	// set items
	for i, ls := range vals {
		activePorts := make([]string, 0)

		port := util.VTrim(ls, idxPort)
		if port != "" {
			ls := strings.SplitSeq(port, ",")
			for v := range ls {
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

		usePort := strings.Join(activePorts, ", ")
		if portContains != "" {
			if !strings.Contains(usePort, portContains) {
				continue
			}
		}

		items = append(items, []any{
			i + 1,
			util.VTrim(ls, idxName),
			util.VTrim(ls, idxStatus),
			usePort,
			util.VTrim(ls, idxImage),
		})
	}

	// add header
	items = slices.Insert(items, 0, []any{"", "NAMES", "STATUS", "PORTS", "IMAGE"})
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
