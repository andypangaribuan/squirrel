/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package taskfile

import (
	"fmt"
	"squirrel/arg"
	"squirrel/util"

	"github.com/wissance/stringFormatter"
)

func CLI() {
	moreInfoMessage := "run 'sq taskfile --help' for more information"
	helpMessage := stringFormatter.FormatComplex(`
info : execute taskfile cli
usage: sq taskfile

{options}
  --file   [+value] path of .taskfile (default current directory)
`, map[string]any{})

	isOptHelp, index := arg.Search("--help")
	arg.Remove(index)

	filePath := arg.GetOptValue(moreInfoMessage, "--file")

	if arg.Count() > 0 {
		util.UnknownCommand(arg.Remains(), moreInfoMessage)
	}

	if isOptHelp {
		util.PrintThenExit(helpMessage)
	}

	execTaskfile(filePath)
}

func execTaskfile(filePath string) {
	var (
		filePaths = []string{fmt.Sprintf("%v/.taskfile", getWorkingDirectory())}
		model     = &stuTaskfile{
			items:            make([][]any, 0),
			newLineAtIndexes: make([]int, 0),
		}
	)

	if filePath != "" {
		filePaths = append(filePaths, filePath)
	}

	for _, filePath := range filePaths {
		err := fileOutput(filePath, model)
		if err != nil {
			return
		}
	}

	printOutput(model)
}
