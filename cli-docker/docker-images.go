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
	"fmt"
	"os"
	"slices"
	"squirrel/util"
	"strings"
)

func dockerImages(optContains string, printOutput ...bool) string {
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
	)

	// set items
	for _, ls := range vals {
		var (
			valRepository = util.VTrim(ls, idxRepository)
			valTag        = util.VTrim(ls, idxTag)
			valImg        = fmt.Sprintf("%v:%v", valRepository, valTag)
		)

		if optContains != "" {
			if !strings.Contains(valImg, optContains) {
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

	// add header
	items = slices.Insert(items, 0, []any{"IMAGE", "TAG", "CREATED", "SIZE"})
	output := util.Build(items)

	if len(printOutput) > 0 && printOutput[0] {
		fmt.Println(output)
	}

	return output
}
