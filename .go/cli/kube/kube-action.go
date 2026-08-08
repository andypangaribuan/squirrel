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
	"squirrel/arg"
	"squirrel/util"

	"github.com/andypangaribuan/gmod/fm"
	"github.com/wissance/stringFormatter"
)

func cliKubeAction() {
	moreInfoMessage := "run 'sq kube action --help' for more information"
	helpMessage := stringFormatter.FormatComplex(`
info : comprehensive kubectl execution
usage: sq kube action

{required-options}
  --app            [+value] application name, when value start with {kyml} then get from .env file
  --yml            [+value|{csv}] execution of yaml file
                   values: {ymls}

{options}
  --namespace      [+value] application namespace, when value start with {kyml} then get from .env file
  --yml-template   [+value|{csv}] last yml file used when --yml not found
                   e.q. {yml-template}={sa1},{svc1}
                   try-1: search from current directory up to 4 level above
                   try-2: search from os environment {template-dir} (directory path)
                          [os env] {export}
                   yml file inside directory: {sa2}, {svc2}
                   try-3: search to github repo {github-repo}
  --verbose        show full message on action
`, map[string]any{
		"required-options": util.ColorBoldGreen("required-options:"),
		"options":          util.ColorBoldGreen("options:"),
		"ymls":             util.ColorYellow("sa, cm, secret, dep, pdb, hpa, svc, ing, stateful, pv, pvc"),
		"kyml":             util.ColorYellow("KYML_"),
		"template-dir":     util.ColorYellow("'SQ_CLI_TEMPLATE_DIR'"),
		"export":           util.ColorCyan("export SQ_CLI_TEMPLATE_DIR=/path/to/your/template/directory"),
		"yml-template":     util.ColorYellow("--yml-template"),
		"csv":              util.ColorYellow("csv"),
		"sa1":              util.ColorBoldRed("sa"),
		"sa2":              util.ColorBoldRed("sa.yml"),
		"svc1":             util.ColorBoldRed("svc-rest"),
		"svc2":             util.ColorBoldRed("svc-rest.yml"),
		"github-repo":      util.ColorBoldGreen("https://github.com/andypangaribuan/squirrel/tree/main/template"),
	})

	isOptHelp, index := arg.Search("--help")
	arg.Remove(index)

	isVerbose, index := arg.Search("--verbose")
	arg.Remove(index)

	envs := getEnvs(getWorkingDirectory())
	appName := arg.GetOptValue(moreInfoMessage, "--app")
	ymls := util.Split(arg.GetOptValue(moreInfoMessage, "--yml"), ",")
	namespace := arg.GetOptValue(moreInfoMessage, "--namespace")
	ymlTemplates := util.Split(arg.GetOptValue(moreInfoMessage, "--yml-template"), ",")

	if len(appName) > 5 && appName[:5] == "KYML_" {
		val, ok := envs[appName]
		if ok {
			appName = val
		}
	}

	if len(namespace) > 5 && namespace[:5] == "KYML_" {
		val, ok := envs[namespace]
		if ok {
			namespace = val
		}
	}

	if isOptHelp {
		util.PrintThenExit(helpMessage)
	}

	if appName == "" || len(ymls) == 0 {
		util.UnknownCommand("", moreInfoMessage)
	}

	if arg.Count() == 0 {
		execKubeActionShow(isVerbose, ymls)
	}

	for _, key := range []string{"apply", "yml", "diff", "delete"} {
		isCommand, index := arg.Search(key)
		if isCommand && index == 0 {
			arg.Remove(index)

			optValue := arg.Get(index)
			arg.Remove(index)

			if optValue == "" || !fm.IfHaveIn(optValue, ymls...) || arg.Count() > 0 {
				util.UnknownCommand(arg.Remains(), moreInfoMessage)
			}

			switch key {
			case "apply":
				execKubeActionApply(optValue, ymlTemplates)
			case "yml":
				execKubeActionYml(optValue, ymlTemplates)
			case "diff":
				execKubeActionDiff(optValue, ymlTemplates)
			case "delete":
				execKubeActionDelete(optValue, ymlTemplates)
			}
		}
	}

	for _, key := range []string{"conf", "secret"} {
		isCommand, index := arg.Search(key)
		if isCommand && index == 0 {
			arg.Remove(index)

			if arg.Count() > 0 {
				util.UnknownCommand(arg.Remains(), moreInfoMessage)
			}

			switch key {
			case "conf":
				execKubeActionConf(namespace, appName, ymls)
			case "secret":
				execKubeActionSecret(namespace, appName)
			}
		}
	}

	isPods, index := arg.Search("pods")
	arg.Remove(index)
	if isPods {
		execKubeActionPods(namespace, appName)
	}

	if arg.Count() > 0 {
		util.UnknownCommand(arg.Remains(), moreInfoMessage)
	}
}
