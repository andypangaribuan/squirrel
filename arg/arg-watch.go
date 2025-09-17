/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package arg

import (
	"fmt"
	"squirrel/util"
)

func Watch(currentPath string, helpMessage string) *stuWatch {
	return &stuWatch{
		currentPath: currentPath,
		helpMessage: helpMessage,
		items:       make([]*stuWatchItem, 0),
	}
}

func (slf *stuWatch) Add(name string, alias string, callback func()) *stuWatch {
	slf.items = append(slf.items, &stuWatchItem{
		name:     name,
		alias:    alias,
		callback: callback,
	})

	return slf
}

func (slf *stuWatch) Exec() {
	var (
		wi      *stuWatchItem
		av      = Get()
		isError = true
	)

	if av == "--help" {
		util.PrintThenExit(slf.helpMessage)
		return
	}

	if av != "" {
		for _, item := range slf.items {
			if item.name == av {
				wi = item
				break
			}

			if item.alias == av {
				wi = item
				break
			}
		}
	}

	if wi != nil {
		isError = false

		if !isError {
			Remove()
		}
	}

	if isError {
		help := fmt.Sprintf("run '%v --help' for more information", slf.currentPath)
		util.UnknownCommand(Remains(), help)
		return
	}

	wi.callback()
}
