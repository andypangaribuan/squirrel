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
	"squirrel/util"
	"strings"
)

func Get(index ...int) string {
	if len(args) == 0 {
		return ""
	}

	if len(index) > 0 {
		if len(args) <= index[0] {
			return ""
		}
		return args[index[0]]
	}

	return args[0]
}

func Search(name string, alias ...string) (bool, int) {
	if len(args) == 0 {
		return false, -1
	}

	aliasName := ""
	if len(alias) > 0 {
		aliasName = alias[0]
	}

	for i, v := range args {
		if v == name || v == aliasName {
			return true, i
		}
	}

	return false, -1
}

func GetOptValue(moreInfoMessage string, name string, alias ...string) string {
	value := ""
	index := -1
	aliasName := ""
	if len(alias) > 0 {
		aliasName = alias[0]
	}

	condition := func(key string, i int, v string) bool {
		if len(v) >= len(key) && v[:len(key)] == key {
			if v == key {
				index = i
				return true
			}

			key += "="
			if len(v) >= len(key) && v[:len(key)] == key {
				index = i
				value = v[len(key):]
				return true
			}
		}

		return false
	}

	for i, v := range args {
		found := condition(name, i, v)
		if found {
			break
		}

		if aliasName != "" {
			found = condition(aliasName, i, v)
			if found {
				break
			}
		}
	}

	if index == -1 {
		return ""
	}

	if value == "" {
		value = Get(index + 1)
		if value == "" {
			util.UnknownCommand(Remains(), moreInfoMessage)
		}

		Remove(index + 1)
	}

	Remove(index)
	return value
}

func Remove(atIndex ...int) {
	if len(args) == 0 {
		return
	}

	index := 0
	if len(atIndex) > 0 {
		index = atIndex[0]
	}

	if index < 0 || index >= len(args) {
		return
	}

	if index == 0 {
		args = args[1:]
	} else {
		args = append(args[:index], args[index+1:]...)
	}
}

func Remains() string {
	if len(args) == 0 {
		return ""
	}

	return strings.Join(args, " ")
}
