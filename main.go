/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package main

import (
	"fmt"
	"squirrel/arg"
	"squirrel/cli/docker"
	"squirrel/cli/kube"
	"squirrel/cli/taskfile"
	"squirrel/util"

	_ "github.com/andypangaribuan/gmod"
	"github.com/wissance/stringFormatter"
)

const version = "2.0.0"

func main() {
	util.ExitWithCtrlC()
	arg.Init()

	helpMessage := stringFormatter.FormatComplex(`
usage: sq

{commands}
  docker     execute docker cli
  kube       execute kubectl cli
  taskfile   execute taskfile cli
  version    print sq-cli version
`, map[string]any{
		"commands": util.ColorBoldGreen("commands:"),
	})

	arg.Watch("sq", helpMessage, helpMessage).
		Add("docker", "", docker.CLI).
		Add("kube", "", kube.CLI).
		Add("taskfile", "", taskfile.CLI).
		Add("version", "", func() { fmt.Printf("version %v\n", version) }).
		Exec()
}
