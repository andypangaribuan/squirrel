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
	"squirrel/util"

	"github.com/wissance/stringFormatter"
)

func kubeAction(lsYml []string) {
	ys := ""
	for _, v := range lsYml {
		if ys != "" {
			ys += ", "
		}
		ys += v
	}

	util.PrintHelp(stringFormatter.FormatComplex(`
{commands}
  apply         Apply yml configuration   : {ys}
  yml           Show content of yml file  : {ys}
  diff          Compare yml configuration : {ys}
  delete        Delete yml configuration  : {ys}

  conf          Show all configurations
  secret        Show all decoded secret
  exec {name}   Go to shell pod (default: first pod)
  pods          Execute pods cli

{pods-subcommand}
  ls              Show running pods
  watch           Stream every second of running pods
  rollout         Rolling update of application
  delete {name1}   Delete specific pod
  scale {size}    Scale deployment to [int] size
  logs {since}    Stream pods log, (default) since: 60m
  events          Stream pods events
`, map[string]any{
		"ys":              util.ColorYellow(ys),
		"commands":        util.ColorBoldGreen("commands:"),
		"pods-subcommand": util.ColorYellow("[") + util.ColorBoldRed("pods") + util.ColorYellow("]") + " " + util.ColorBoldGreen("subcommands:"),
		"name1":           util.ColorCyan("{name}"),
		"size":            util.ColorCyan("{size}"),
	}), false)
}
