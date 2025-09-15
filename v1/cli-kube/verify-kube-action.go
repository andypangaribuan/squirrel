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
	"squirrel/app"
	"squirrel/util"
	"strconv"
	"strings"

	"github.com/andypangaribuan/gmod/fm"
)

func verifyKubeAction() (namespace string, appName string, lsYml []string, lsYmlTemplate []string) {
	lsYml = make([]string, 0)
	lsYmlTemplate = make([]string, 0)

	var (
		isError    = false
		args       = app.Args
		workingDir = getWorkingDirectory()
		envs       = getEnvs(workingDir)
		command    = "kube action"
		remains    = args.GetRemains(command, "--help", "--watch")
		optKey     string
		optKeyVal  string
		optVal     string
	)

	if remains == "" && args.IsOptHelp {
		return
	}

	defer func() {
		if isError {
			switch {
			case args.IsApply && remains == "apply",
				args.IsYml && remains == "yml",
				args.IsDiff && remains == "diff",
				args.IsDelete && remains == "delete",
				args.IsConf && remains == "conf",
				args.IsSecret && remains == "secret",
				args.IsExec && remains == "exec",
				args.IsPods && remains == "pods":
				remains = ""
			}

			util.UnknownCommand(remains, "run 'sq kube action --help' for more information")
		}
	}()

	exit := func() (string, string, []string, []string) {
		isError = true
		return namespace, appName, lsYml, lsYmlTemplate
	}

	removeOptKeyVal := func() {
		remains = strings.TrimSpace(strings.Replace(remains, optKeyVal, "", 1))
	}

	// verify: --namespace
	optKey, optKeyVal, optVal = args.GetOptVal(remains, "--namespace")
	if optKey == "" {
		return exit()
	} else {
		if optVal == "" {
			return exit()
		}

		namespace = optVal
		if len(namespace) > 5 && namespace[:5] == "KYML_" {
			if val, ok := envs[namespace]; ok {
				namespace = val
			}
		}

		removeOptKeyVal()
	}

	// verify: --app
	optKey, optKeyVal, optVal = args.GetOptVal(remains, "--app")
	if optKey == "" {
		return exit()
	} else {
		if optVal == "" {
			return exit()
		}

		appName = optVal
		if len(appName) > 5 && appName[:5] == "KYML_" {
			if val, ok := envs[appName]; ok {
				appName = val
			}
		}

		removeOptKeyVal()
	}

	// verify: --yml
	optKey, optKeyVal, optVal = args.GetOptVal(remains, "--yml")
	if optKey != "" && optVal == "" {
		return exit()
	}

	if optVal != "" {
		ls := strings.Split(optVal, ",")
		for _, v := range ls {
			v = strings.TrimSpace(v)
			if !fm.IfHaveIn(v, availableYml...) {
				return exit()
			}

			lsYml = append(lsYml, v)
		}
	}

	if len(lsYml) == 0 {
		isExit := true
		if args.IsPods && args.IsEvents && util.ReplaceDoubleSpaceToSingleSpace(remains) == "pods events" {
			isExit = false
		}

		if isExit {
			return exit()
		}
	}

	if optKeyVal != "" {
		removeOptKeyVal()
	}

	// verify: --yml-template
	optKey, optKeyVal, optVal = args.GetOptVal(remains, "--yml-template")
	if optKey != "" && optVal == "" {
		return exit()
	}

	if optVal != "" {
		ls := strings.Split(optVal, ",")
		for _, ymlTemplate := range ls {
			isHave := false
			ymlTemplate = strings.TrimSpace(ymlTemplate)

			for _, yml := range availableYml {
				if ymlTemplate == yml {
					isHave = true
					break
				}

				if len(ymlTemplate) > len(yml) && ymlTemplate[:len(yml)] == yml {
					nextChar := ymlTemplate[len(yml) : len(yml)+1]
					if nextChar == "-" {
						isHave = true
						break
					}
				}
			}

			if !isHave {
				return exit()
			}

			lsYmlTemplate = append(lsYmlTemplate, ymlTemplate)
		}

		removeOptKeyVal()
	}

	// verify: apply
	optKey, optKeyVal, optVal = args.GetOptVal(remains, "apply")
	if optKey != "" {
		if optVal == "" {
			return exit()
		}

		if !fm.IfHaveIn(optVal, availableYml...) {
			return exit()
		}

		removeOptKeyVal()
		if remains != "" {
			return exit()
		}
	}

	// verify: yml
	optKey, optKeyVal, optVal = args.GetOptVal(remains, "yml")
	if optKey != "" {
		if optVal == "" {
			return exit()
		}

		if !fm.IfHaveIn(optVal, availableYml...) {
			return exit()
		}

		removeOptKeyVal()
		if remains != "" {
			return exit()
		}
	}

	// verify: diff
	optKey, optKeyVal, optVal = args.GetOptVal(remains, "diff")
	if optKey != "" {
		if optVal == "" {
			return exit()
		}

		if !fm.IfHaveIn(optVal, availableYml...) {
			return exit()
		}

		removeOptKeyVal()
		if remains != "" {
			return exit()
		}
	}

	// verify: delete
	if len(remains) > 5 && remains[:5] != "pods"+singleSpace {
		optKey, optKeyVal, optVal = args.GetOptVal(remains, "delete")
		if optKey != "" {
			if optVal == "" {
				return exit()
			}

			if !fm.IfHaveIn(optVal, availableYml...) {
				return exit()
			}

			removeOptKeyVal()
			if remains != "" {
				return exit()
			}
		}
	}

	// verify: conf
	if args.IsConf {
		optKeyVal = "conf"
		removeOptKeyVal()
	}

	// verify: secret
	if args.IsSecret {
		optKeyVal = "secret"
		removeOptKeyVal()
	}

	if remains != "" {
		if args.IsPods {
			optKeyVal = "pods"
			removeOptKeyVal()

			if args.IsLs {
				optKeyVal = "ls"
				removeOptKeyVal()
			}

			if args.IsWatch {
				optKeyVal = "watch"
				removeOptKeyVal()
			}

			if args.IsRollout {
				optKeyVal = "rollout"
				removeOptKeyVal()
			}

			if args.IsDelete {
				_, optKeyVal, optVal = args.GetOptVal(remains, "delete")
				if optVal == "" {
					return exit()
				}

				removeOptKeyVal()
			}

			if args.IsScale {
				_, optKeyVal, optVal = args.GetOptVal(remains, "scale")
				if optVal == "" {
					return exit()
				}

				_, err := strconv.Atoi(optVal)
				if err != nil {
					return exit()
				}

				removeOptKeyVal()
			}

			if args.IsLogs {
				_, optKeyVal, _ = args.GetOptVal(remains, "logs")
				removeOptKeyVal()
			}

			if args.IsEvents {
				optKeyVal = "events"
				removeOptKeyVal()
			}
		}
	}

	if remains != "" {
		return exit()
	}

	return
}
