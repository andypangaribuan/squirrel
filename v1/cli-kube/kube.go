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
)

func Exec() {
	args := app.Args

	switch {
	case args.IsInfo:
		namespace, appNames := verifyKubeInfo()
		if args.IsOptHelp {
			util.PrintHelp(msgHelpKubeInfo, false)
		}

		if args.IsOptWatch {
			util.Watch(func() string {
				return kubeInfo(namespace, appNames)
			})
		}

		kubeInfo(namespace, appNames, true)

	case args.IsAction:
		namespace, appName, lsYml, lsYmlTemplate := verifyKubeAction()
		if args.IsOptHelp {
			util.PrintHelp(msgHelpKubeAction, false)
		}

		if args.IsApply {
			kubeActionApply(lsYml, lsYmlTemplate)
			return
		}

		if args.IsYml {
			kubeActionYml(lsYml, lsYmlTemplate)
			return
		}

		if args.IsDiff {
			kubeActionDiff(lsYml, lsYmlTemplate)
			return
		}

		if args.IsDelete && !args.IsPods {
			kubeActionDelete(lsYml, lsYmlTemplate)
			return
		}

		if args.IsConf {
			kubeActionConf(namespace, appName, lsYml)
			return
		}

		if args.IsSecret {
			kubeActionSecret(namespace, appName)
			return
		}

		if args.IsPods {
			kubeActionPods(namespace, appName)
			return
		}

		kubeAction(lsYml)

	case args.IsOptHelp:
		util.PrintHelp(msgHelpKube)

	default:
		util.UnknownCommand("", "run 'sq kube --help' for more information")
	}
}
