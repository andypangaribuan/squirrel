/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package util

import (
	"os"
	"squirrel/app"
	"squirrel/model"
	"strings"
)

func ArgsExtractor() *model.Args {
	osArgs := os.Args[1:]
	for i, v := range osArgs {
		v = strings.TrimSpace(v)
		if v != "" {
			osArgs[i] = v
		}
	}

	args := &model.Args{}
	args.SetArgs(osArgs)

	for _, v := range osArgs {
		switch v {
		case "--help":
			args.IsOptHelp = true

		case "--watch":
			args.IsOptWatch = true

		case "version":
			args.IsVersion = true

		case "docker":
			args.IsDocker = true

		case "ps":
			args.IsPs = true

		case "images":
			args.IsImages = true

		case "kube":
			args.IsKube = true

		case "info":
			args.IsInfo = true

		case "action":
			args.IsAction = true

		case "apply":
			args.IsApply = true

		case "yml":
			args.IsYml = true

		case "diff":
			args.IsDiff = true

		case "delete":
			args.IsDelete = true

		case "conf":
			args.IsConf = true

		case "secret":
			args.IsSecret = true

		case "exec":
			args.IsExec = true

		case "pods":
			args.IsPods = true

		case "ls":
			args.IsLs = true

		case "watch":
			args.IsWatch = true

		case "rollout":
			args.IsRollout = true

		case "logs":
			args.IsLogs = true

		case "events":
			args.IsEvents = true
		}
	}

	app.Args = args
	return args
}
