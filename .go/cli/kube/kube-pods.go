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
	"fmt"
	"squirrel/arg"
	"squirrel/util"
	"sync"

	"github.com/wissance/stringFormatter"
)

func cliKubePods() {
	moreInfoMessage := "run 'sq kube pods --help' for more information"
	helpMessage := stringFormatter.FormatComplex(`
info : show pods information
usage: sq kube pods {deploy-name|ssv}

{options}
  -n, --namespace   [+value] deploy namespace
      --watch       stream every second
`, map[string]any{
		"options": util.ColorBoldGreen("options:"),
	})

	isOptHelp, index := arg.Search("--help")
	arg.Remove(index)

	isOptWatch, index := arg.Search("--watch")
	arg.Remove(index)

	namespace := arg.GetOptValue(moreInfoMessage, "-n", "--namespace")
	deployNames := make([]string, 0)

	for {
		name := arg.Get(0)
		arg.Remove(0)
		if name == "" {
			break
		}

		deployNames = append(deployNames, name)
	}

	if isOptHelp {
		util.PrintThenExit(helpMessage)
	}

	if len(deployNames) == 0 {
		util.UnknownCommand(arg.Remains(), moreInfoMessage)
	}

	if isOptWatch {
		util.Watch(func() string {
			return execKubePods(namespace, deployNames)
		})
	}

	execKubePods(namespace, deployNames, true)
}

func execKubePods(namespace string, deployNames []string, doPrint ...bool) string {
	var (
		wg     sync.WaitGroup
		output string
		lsOut  = make([]string, len(deployNames))
	)

	wg.Add(len(deployNames))
	for i, appName := range deployNames {
		go func() {
			out := getInfoPods(namespace, appName)
			lsOut[i] = out
			wg.Done()
		}()
	}

	wg.Wait()
	for _, out := range lsOut {
		if output != "" {
			output += "\n\n\n\n"
		}
		output += out
	}

	if len(doPrint) > 0 && doPrint[0] {
		fmt.Println(output)
	}

	return output
}
