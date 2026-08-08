/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package docker

import (
	"fmt"
	"os"
	"slices"
	"squirrel/arg"
	"squirrel/model"
	"squirrel/util"
	"strings"

	"github.com/wissance/stringFormatter"
)

func cliDockerImages() {
	moreInfoMessage := "run 'sq docker images --help' for more information"
	helpMessage := stringFormatter.FormatComplex(`
info : list docker image
usage: sq docker images

{options}
  --contains   [+value] filter by image name
  --order      [+value|csv] order by columns name
`, map[string]any{
		"options": util.ColorBoldGreen("options:"),
	})

	isOptHelp, index := arg.Search("--help")
	arg.Remove(index)

	contains := arg.GetOptValue(moreInfoMessage, "--contains")
	orderBy := arg.GetOptValue(moreInfoMessage, "--order")

	if arg.Count() > 0 {
		util.UnknownCommand(arg.Remains(), moreInfoMessage)
	}

	if isOptHelp {
		util.PrintThenExit(helpMessage)
	}

	execDockerImages(contains, strings.Split(orderBy, ","))
}

func execDockerImages(contains string, orderColumns []string) {
	out, err := util.Terminal("", "docker images")
	if err != nil {
		fmt.Println(*err)
		os.Exit(1)
	}

	keys, vals := util.MapKV(out, "REPOSITORY", "TAG", "IMAGE ID", "CREATED", "SIZE")
	var (
		items         = make([][]any, 0)
		idxRepository = keys["REPOSITORY"]
		idxTag        = keys["TAG"]
		idxCreated    = keys["CREATED"]
		idxSize       = keys["SIZE"]
		orderBy       = make([]model.OrderColumn, 0)
	)

	for _, c := range orderColumns {
		switch strings.ToLower(c) {
		case "image":
			orderBy = append(orderBy, model.OrderColumn{
				Index: 0,
			})

		case "tag":
			orderBy = append(orderBy, model.OrderColumn{
				Index: 1,
			})

		case "created":
			orderBy = append(orderBy, model.OrderColumn{
				Index: 2,
			})

		case "size":
			orderBy = append(orderBy, model.OrderColumn{
				Index: 3,
			})
		}
	}

	// set items
	for _, ls := range vals {
		var (
			valRepository = util.VTrim(ls, idxRepository)
			valTag        = util.VTrim(ls, idxTag)
			valImg        = fmt.Sprintf("%v:%v", valRepository, valTag)
		)

		if contains != "" {
			if !strings.Contains(valImg, contains) {
				continue
			}
		}

		items = append(items, []any{
			valImg,
			valRepository,
			valTag,
			util.VTrim(ls, idxCreated),
			util.VTrim(ls, idxSize),
		})
	}

	for i := range items {
		items[i] = items[i][1:]
	}

	if len(orderBy) > 0 {
		util.SortRows(items, orderBy)
	}

	// add header
	items = slices.Insert(items, 0, []any{"IMAGE", "TAG", "CREATED", "SIZE"})
	output := util.Build(items)

	fmt.Println(output)
}
