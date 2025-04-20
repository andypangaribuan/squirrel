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
	"os"
	clidocker "squirrel/cli-docker"
	clikube "squirrel/cli-kube"
	"squirrel/util"

	_ "github.com/andypangaribuan/gmod"
	"github.com/wissance/stringFormatter"
)

const version = "1.0.2"

var msgHelp string

func init() {
	msgHelp = stringFormatter.FormatComplex(`
usage: sq [commands]

{commands}
  docker    Execute docker cli
  kube      Execute kubectl cli
  version   Print sq-cli version
`, map[string]any{
		"commands": util.ColorBoldGreen("commands:"),
	})
}

func main() {
	util.ExitWithCtrlC()
	args := util.ArgsExtractor()

	switch {
	case args.IsVersion:
		fmt.Printf("version %v\n", version)
		os.Exit(0)

	case args.IsDocker:
		clidocker.Exec()

	case args.IsKube:
		clikube.Exec()

	case args.IsOptHelp:
		util.PrintHelp(msgHelp, false)

	default:
		util.UnknownCommand(args.Command, "run 'sq --help' for more information")
	}
}
