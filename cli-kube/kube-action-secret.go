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
	"encoding/base64"
	"fmt"
	"os"
	"squirrel/util"
	"strings"

	"github.com/andypangaribuan/gmod/fm"
	"github.com/andypangaribuan/gmod/gm"
)

func kubeActionSecret(namespace string, appName string) {
	var (
		output            string
		noResourceMessage = "no resources found"
		emptyMessage      = util.ColorCyan("<empty>")
		script            = fmt.Sprintf("kubectl get secret --field-selector metadata.name=%v -n %v -o json", appName, namespace)
	)

	out, errMsg := util.Terminal("", script)
	if errMsg != nil {
		if !strings.Contains(strings.ToLower(*errMsg), noResourceMessage) {
			fmt.Println(*errMsg)
			os.Exit(1)
		}

		fmt.Printf("%v\n\n", util.ColorCyan("no secret found"))
		os.Exit(1)
	}

	type stuSecret struct {
		Items []struct {
			Data map[string]string `json:"data"`
		} `json:"items"`
	}

	var model *stuSecret
	err := gm.Json.Decode(out, &model)
	if err != nil {
		fmt.Printf("%+v\n\n", err)
		os.Exit(1)
	}

	if model == nil {
		fmt.Printf("%v\n\n", util.ColorCyan("error on json decode process"))
		os.Exit(1)
	}

	getSeparator := func(key string) string {
		sep := ""
		for len(key) > len(sep) {
			sep += "-"
		}
		return sep
	}

	for _, item := range model.Items {
		for key, encoded := range item.Data {
			data, err := base64.StdEncoding.DecodeString(encoded)
			if err != nil {
				fmt.Printf("%v\n%+v\n\n", util.ColorCyan("error on base64 decode process"), err)
				os.Exit(1)
			}

			if output != "" {
				output += "\n\n"
			}

			value := strings.TrimSpace(string(data))
			output += util.ColorBoldGreen(key) + "\n" +
				util.ColorBoldGreen(getSeparator(key)) + "\n" +
				fm.Ternary(value == "", emptyMessage, value)
		}
	}

	fmt.Printf("%v\n\n", output)
	os.Exit(0)
}
