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
	"sort"
	"squirrel/app"
	"squirrel/arg"
	"squirrel/util"
	"strconv"
	"strings"
	"sync"

	"github.com/andypangaribuan/gmod/fm"
	"github.com/wissance/stringFormatter"
)

func execKubeActionPods(namespace string, appName string) {
	moreInfoMessage := "run 'sq kube action pods --help' for more information"
	helpMessage := stringFormatter.FormatComplex(`
{commands}
{items}
`, map[string]any{
		"commands": util.ColorBoldGreen("commands:"),
		"items":    util.TwoCenter(commandActionPods, doubleSpace, tripleSpace, -1),
	})

	if arg.Count() == 0 {
		util.PrintThenExit(helpMessage)
	}

	for _, key := range []string{"ls", "watch", "rollout"} {
		isCommand, index := arg.Search(key)
		if isCommand && index == 0 {
			arg.Remove(index)
			if arg.Count() > 0 {
				util.UnknownCommand(arg.Remains(), moreInfoMessage)
			}

			switch key {
			case "ls":
				execKubeActionPodsLs(namespace, appName)
			case "watch":
				execKubeActionPodsWatch(namespace, appName)
			case "rollout":
				execKubeActionPodsRollout(namespace, appName)
			}
		}
	}

	for _, key := range []string{"delete", "exec", "scale", "logs"} {
		isCommand, index := arg.Search(key)
		if isCommand && index == 0 {
			arg.Remove(index)

			av := arg.Get(0)
			arg.Remove(0)

			if arg.Count() > 0 {
				util.UnknownCommand(arg.Remains(), moreInfoMessage)
			}

			switch key {
			case "delete":
				execKubeActionPodsDelete(namespace, appName, av)
			case "exec":
				execKubeActionPodsExec(namespace, appName, av)
			case "logs":
				execKubeActionPodsLogs(namespace, appName, av)

			case "scale":
				scale := 0
				if av != "" {
					v, err := strconv.Atoi(av)
					if err == nil {
						scale = v
					}
				}

				if scale <= 0 {
					util.UnknownCommand(arg.Remains(), moreInfoMessage)
				}
				execKubeActionPodsScale(namespace, appName, scale)
			}
		}
	}

	isEvents, index := arg.Search("events")
	if isEvents && index == 0 {
		arg.Remove(index)

		isWatch, index := arg.Search("--watch")
		arg.Remove(index)

		if arg.Count() > 0 {
			util.UnknownCommand(arg.Remains(), moreInfoMessage)
		}
		execKubeActionPodsEvents(namespace, appName, isWatch)
	}
}

func execKubeActionPodsLs(namespace string, appName string) {
	deployNames := []string{appName}
	execKubePods(namespace, deployNames, true)
}

func execKubeActionPodsWatch(namespace string, appName string) {
	command := fmt.Sprintf(`watch -t -n 1 "%v kube pods %v"`, app.SqCli, appName)
	if namespace != "" {
		command += " --namespace=" + namespace
	}

	err := util.InteractiveTerminal("", command)
	if err != nil {
		fmt.Printf("%+v\n", err)
		os.Exit(1)
	}
}

func execKubeActionPodsRollout(namespace string, appName string) {
	command := "kubectl rollout restart deploy " + appName
	if namespace != "" {
		command += " -n " + namespace
	}

	out, err := util.Terminal("", command)
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

func execKubeActionPodsExec(namespace string, appName string, podName string) {
	if podName != "" {
		podName = appName + "-" + podName
	} else {
		command := "kubectl get pod -l app=" + appName
		if namespace != "" {
			command += " -n " + namespace
		}

		out, err := util.Terminal("", command)
		if err != nil {
			fmt.Println(*err)
			os.Exit(1)
		}

		keys, vals := util.MapKV(out, "NAME", "READY", "STATUS", "RESTARTS", "AGE")
		if len(vals) == 0 {
			return
		}

		idxName := keys["NAME"]
		sort.SliceStable(vals, func(i, j int) bool {
			return vals[i][idxName] < vals[j][idxName]
		})

		podName = vals[0][idxName]
	}

	var (
		haveSH        = false
		haveBash      = false
		terminalShell = ""
	)

	if !haveSH {
		command := "kubectl exec " + podName + " -c " + appName
		command += fm.Ternary(namespace == "", "", " -n "+namespace)
		command += " -- which sh"
		_, err := util.Terminal("", command)
		if err == nil {
			haveSH = true
		}
	}

	if !haveBash {
		command := "kubectl exec " + podName + " -c " + appName
		command += fm.Ternary(namespace == "", "", " -n "+namespace)
		command += " -- which bash"
		_, err := util.Terminal("", command)
		if err == nil {
			haveBash = true
		}
	}

	switch {
	case haveBash:
		terminalShell = "bash"
	case haveSH:
		terminalShell = "sh"
	}

	if terminalShell != "" {
		command := "kubectl exec -it " + podName + " -c " + appName
		command += fm.Ternary(namespace == "", "", " -n "+namespace)
		command += " -- " + terminalShell
		_ = util.InteractiveTerminal("", command)
	}
}

func execKubeActionPodsDelete(namespace string, appName string, podName string) {
	if podName != "" {
		podName = appName + "-" + podName
	} else {
		command := "kubectl get pod -l app=" + appName
		if namespace != "" {
			command += " -n " + namespace
		}

		out, err := util.Terminal("", command)
		if err != nil {
			fmt.Println(*err)
			os.Exit(1)
		}

		keys, vals := util.MapKV(out, "NAME", "READY", "STATUS", "RESTARTS", "AGE")
		if len(vals) == 0 {
			return
		}

		idxName := keys["NAME"]
		sort.SliceStable(vals, func(i, j int) bool {
			return vals[i][idxName] < vals[j][idxName]
		})

		podName = vals[0][idxName]
	}

	command := "kubectl delete pods " + podName
	if namespace != "" {
		command += " -n " + namespace
	}

	out, err := util.Terminal("", command)
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

func execKubeActionPodsScale(namespace string, appName string, scale int) {
	command := fmt.Sprintf("kubectl scale --replicas=%v deploy/%v", scale, appName)
	if namespace != "" {
		command += " -n " + namespace
	}

	out, err := util.Terminal("", command)
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

func execKubeActionPodsLogs(namespace string, appName string, since string) {
	if since == "" {
		since = "60m"
	}

	base := "stern"
	if namespace != "" {
		base += " -n " + namespace
	}

	err := util.InteractiveTerminal("",
		stringFormatter.FormatComplex(
			`{base} {app} -c {app} -l app={app} -t --since {since}`,
			map[string]any{
				"base":  base,
				"app":   appName,
				"since": since,
			}))

	if err != nil {
		fmt.Printf("%+v\n", err)
		os.Exit(1)
	}
}

func execKubeActionPodsEvents(namespace string, appName string, isWatch bool) {
	if isWatch {
		command := app.SqCli + " kube action"
		command += " --app " + appName
		command += " --yml dep"
		command += fm.Ternary(namespace == "", "", " --namespace "+namespace)
		command += " pods events"

		err := util.InteractiveTerminal("", fmt.Sprintf(`watch -c -t -n 1 "unbuffer %v"`, command))
		if err != nil {
			fmt.Printf("%+v\n", err)
			os.Exit(1)
		}

		return
	}

	command := "kubectl get pod -n %v -l app=" + appName
	command += fm.Ternary(namespace == "", "", " -n "+namespace)
	out, err := util.Terminal("", command)
	if err != nil {
		fmt.Println(*err)
		os.Exit(1)
	}

	keys, vals := util.MapKV(out, "NAME", "READY", "STATUS", "RESTARTS", "AGE")
	idxName := keys["NAME"]

	sort.SliceStable(vals, func(i, j int) bool {
		return vals[i][idxName] < vals[j][idxName]
	})

	if len(vals) > 0 {
		var (
			wg   sync.WaitGroup
			outs = make(map[int]string, len(vals))
		)

		for index, ls := range vals {
			wg.Add(1)
			podName := ls[idxName]
			go func() {
				command := "kubectl describe pods"
				command += fm.Ternary(namespace == "", "", " -n "+namespace)
				command += " " + podName

				out, err := util.Terminal("", command)
				if err != nil {
					fmt.Println(*err)
					os.Exit(1)
				}

				var (
					lines       = strings.Split(out, "\n")
					foundEvents = false
					eventKey    = "Events:"
					eventValue  = ""
					lastState   = ""
				)

				for index, line := range lines {
					xLine := strings.ToLower(strings.TrimSpace(line))
					if len(xLine) >= 11 && xLine[:11] == "last state:" {
						lastState = "Last State: " + strings.TrimSpace(strings.SplitN(xLine, ":", 2)[1])
						reason := ""
						exitCode := ""

						if len(lines)-1 > index+1 {
							xLine := strings.ToLower(strings.TrimSpace(lines[index+1]))
							if len(xLine) > 7 && xLine[:7] == "reason:" {
								reason = strings.TrimSpace(strings.SplitN(lines[index+1], ":", 2)[1])
							}
						}

						if len(lines)-2 > index+2 {
							xLine := strings.ToLower(strings.TrimSpace(lines[index+2]))
							if len(xLine) > 10 && xLine[:10] == "exit code:" {
								exitCode = strings.TrimSpace(strings.SplitN(lines[index+2], ":", 2)[1])
							}
						}

						if exitCode != "" && reason != "" {
							lastState += fmt.Sprintf(", [%v] %v", exitCode, reason)
						}
					}

					if foundEvents {
						if len(line) > 2 && line[:2] == doubleSpace {
							eventValue += fm.Ternary(eventValue == "", "", "\n") + strings.TrimSpace(line)
							continue
						}
						break
					}

					if len(line) == len(eventKey) && line == eventKey {
						foundEvents = true
					}
				}

				outs[index] = util.ColorBoldGreen(podName) +
					fm.Ternary(lastState == "", "", "\n"+lastState) +
					fm.Ternary(eventValue == "", "", "\n"+eventValue)
				wg.Done()
			}()
		}

		wg.Wait()

		keys := make([]int, 0, len(vals))
		for key := range outs {
			keys = append(keys, key)
		}

		sort.Ints(keys)

		// sort.SliceStable(keys, func(i, j int) bool {
		// 	return len(strings.Split(outs[keys[i]], "\n")) < len(strings.Split(outs[keys[j]], "\n"))
		// })

		output := ""
		for index, key := range keys {
			if index == 0 {
				output = outs[key]
				continue
			}

			if len(strings.Split(outs[keys[index-1]], "\n")) == 1 && len(strings.Split(outs[keys[index]], "\n")) == 1 {
				output += "\n" + outs[key]
			} else {
				output += "\n\n" + outs[key]
			}
		}

		fmt.Printf("%v\n\n", output)
	}
}
