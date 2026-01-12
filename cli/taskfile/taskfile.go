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
	"strings"

	"github.com/wissance/stringFormatter"
)

func CLI() {
	moreInfoMessage := "run 'sq taskfile --help' for more information"
	helpMessage := stringFormatter.FormatComplex(`
info : execute taskfile cli
usage: sq taskfile

{options}
  --file   [+value|csv] path of .taskfile (default current directory)
`, map[string]any{})

	isOptHelp, index := arg.Search("--help")
	arg.Remove(index)

	filePaths := arg.GetOptValue(moreInfoMessage, "--file")

	if arg.Count() > 0 {
		util.UnknownCommand(arg.Remains(), moreInfoMessage)
	}

	if isOptHelp {
		util.PrintThenExit(helpMessage)
	}

	execTaskfile(strings.Split(filePaths, ","))
}

func execTaskfile(ls []string) {
	var (
		filePaths = make([]string, 0)
		model     = &stuTaskfile{
			items:            make([][]any, 0),
			newLineAtIndexes: make([]int, 0),
		}
	)

	currentDotTaskfile := fmt.Sprintf("%v/.taskfile", getWorkingDirectory())
	if util.IsFileExists(currentDotTaskfile) {
		filePaths = append(filePaths, currentDotTaskfile)
	}

	for _, path := range ls {
		if util.IsFileExists(path) {
			filePaths = append(filePaths, path)
		}
	}

	if len(filePaths) == 0 {
		return
	}

	for _, filePath := range filePaths {
		err := fileOutput(filePath, model)
		if err != nil {
			fmt.Printf("error: %+v\n", err)
			return
		}
	}

	printOutput(model)
}
