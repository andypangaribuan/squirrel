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
	"squirrel/app"
	"squirrel/util"
)

func Exec() {
	args := app.Args

	switch {
	case args.IsPs:
		verifyDockerPs()
		if args.IsOptHelp {
			util.PrintHelp(msgHelpDockerPs, false)
		}

		if args.IsOptWatch {
			util.Watch(func() string {
				return dockerPs()
			})
		}

		dockerPs(true)

	case args.IsImages:
		optContains := verifyDockerImages()
		if args.IsOptHelp {
			util.PrintHelp(msgHelpDockerImages, false)
		}

		if args.IsOptWatch {
			util.Watch(func() string {
				return dockerImages(optContains)
			})
		}

		dockerImages(optContains, true)

	case args.IsOptHelp:
		util.PrintHelp(msgHelpDocker, false)

	default:
		var (
			command = "docker"
			remains = args.GetRemains(command, "--help", "--watch")
		)

		util.UnknownCommand(remains, "run 'sq docker --help' for more information")
	}
}
