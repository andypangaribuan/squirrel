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
	"sort"
	"squirrel/app"
	"squirrel/util"
	"strings"
	"sync"

	"github.com/andypangaribuan/gmod/fm"
)

func kubeActionPodsEvents(namespace string, appName string) {
	args := app.Args

	if args.IsOptWatch {
		err := util.InteractiveTerminal("", fmt.Sprintf(
			`watch -c -t -n 1 "unbuffer %v kube action pods --namespace %v --app %v events"`,
			app.SqCli, namespace, appName))

		if err != nil {
			fmt.Printf("%+v\n", err)
			os.Exit(1)
		}

		return
	}

	command := fmt.Sprintf("kubectl get pod -n %v -l app=%v", namespace, appName)
	out, err := util.Terminal("", command)
	if err != nil {
		fmt.Println(*err)
		os.Exit(1)
	}

	keys, vals := util.MapKV(out, "NAME", "READY", "STATUS", "RESTARTS", "AGE")
	idxName := keys["NAME"]

	if len(vals) > 0 {
		var (
			wg   sync.WaitGroup
			outs = make(map[int]string, len(vals))
		)

		for index, ls := range vals {
			wg.Add(1)
			podName := ls[idxName]
			go func() {
				out, err := util.Terminal("", "kubectl describe pods -n %v %v", namespace, podName)
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

		sort.SliceStable(keys, func(i, j int) bool {
			return len(strings.Split(outs[keys[i]], "\n")) < len(strings.Split(outs[keys[j]], "\n"))
		})

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
