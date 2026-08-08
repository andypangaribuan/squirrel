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
	"squirrel/util"
	"strings"

	"github.com/andypangaribuan/gmod/fm"
	"github.com/wissance/stringFormatter"
)

func execKubeActionShow(isVerbose bool, ymls []string) {
	command1Items := make([][]string, 0)
	command2Items := make([][]string, 0)
	command3Items := make([][]string, 0)
	ys := strings.Join(ymls, ", ")

	templateHelpMessage := `
{commands}
  apply, yml, diff, delete  →  {ys}
`

	if isVerbose {
		command1Items = append(command1Items, []string{"apply", "apply yml configuration   : {ys}"})
		command1Items = append(command1Items, []string{"yml", "show content of yml file  : {ys}"})
		command1Items = append(command1Items, []string{"diff", "compare yml configuration : {ys}"})
		command1Items = append(command1Items, []string{"delete", "delete yml configuration  : {ys}"})
	}

	command2Items = append(command2Items, []string{"conf", "show all configurations"})

	if fm.IfHaveIn("secret", ymls...) {
		command2Items = append(command2Items, []string{"secret", "show all decoded secret"})
	}

	if fm.IfHaveIn("dep", ymls...) {
		command2Items = append(command2Items, []string{"pods", "execute pods cli"})
		command3Items = commandActionPods
	}

	forceI0MaxLine := util.CalcI0MaxLine(append(append(command1Items, command2Items...), command3Items...))
	if len(command1Items) > 0 {
		templateHelpMessage = "\n{commands}\n"
		templateHelpMessage += util.TwoCenter(command1Items, doubleSpace, tripleSpace, forceI0MaxLine) + "\n"
	}

	templateHelpMessage += "\n" + util.TwoCenter(command2Items, doubleSpace, tripleSpace, forceI0MaxLine)

	if len(command3Items) > 0 {
		templateHelpMessage += "\n\n{pods-subcommand}"
		templateHelpMessage += "\n" + util.TwoCenter(command3Items, doubleSpace, tripleSpace, forceI0MaxLine)
	}

	helpMessage := stringFormatter.FormatComplex(templateHelpMessage, map[string]any{
		"ys":              util.ColorYellow(ys),
		"commands":        util.ColorBoldGreen("commands:"),
		"pods-subcommand": util.ColorYellow("[") + util.ColorBoldRed("pods") + util.ColorYellow("]") + " " + util.ColorBoldGreen("subcommands:"),
	})

	util.PrintThenExit(helpMessage)
}
