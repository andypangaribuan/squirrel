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
	"os"
	"slices"
	"sort"
	"squirrel/util"
	"strings"
	"sync"
	"time"
)

func getInfoPods(namespace string, appName string) string {
	var (
		wg        sync.WaitGroup
		hpaHeader = make([]any, 0)
		hpaItems  = make([][]any, 0)
		podItems  = make([][]any, 0)
		topItems  = make(map[string][]string, 0)
		imgItems  = make(map[string][]any, 0)
	)

	wg.Add(1)
	go func() {
		command := fmt.Sprintf("kubectl get hpa -n %v %v", namespace, appName)
		if namespace == "" {
			command = fmt.Sprintf("kubectl get hpa %v", appName)
		}

		out, err := util.Terminal("", command)
		if err != nil {
			wg.Done()
			return
		}

		keys, vals := util.MapKV(out, "NAME", "REFERENCE", "TARGETS", "MINPODS", "MAXPODS", "REPLICAS", "AGE")
		var (
			idxName    = keys["NAME"]
			idxTarget  = keys["TARGETS"]
			idxMin     = keys["MINPODS"]
			idxMax     = keys["MAXPODS"]
			idxReplica = keys["REPLICAS"]
		)

		// set header
		hpaHeader = []any{"NAME", "TARGETS", "MIN", "MAX", "REP"}

		// set items
		for _, ls := range vals {
			hpaItems = append(hpaItems, []any{
				util.VTrim(ls, idxName),
				strings.Replace(util.VTrim(ls, idxTarget), "memory", "mem", 1),
				util.VTrim(ls, idxMin),
				util.VTrim(ls, idxMax),
				util.VTrim(ls, idxReplica),
			})
		}

		// hpaOut = util.Build(items)
		wg.Done()
	}()

	wg.Add(1)
	go func() {
		command := fmt.Sprintf("kubectl get pod -n %v -l app=%v", namespace, appName)
		if namespace == "" {
			command = fmt.Sprintf("kubectl get pod -l app=%v", appName)
		}

		out, err := util.Terminal("", command)
		if err != nil {
			fmt.Println(*err)
			os.Exit(1)
		}

		keys, vals := util.MapKV(out, "NAME", "READY", "STATUS", "RESTARTS", "AGE")
		var (
			items      = make([][]any, 0)
			idxName    = keys["NAME"]
			idxReady   = keys["READY"]
			idxStatus  = keys["STATUS"]
			idxRestart = keys["RESTARTS"]
			idxAge     = keys["AGE"]
		)

		// set items
		for _, ls := range vals {
			items = append(items, []any{
				util.VTrim(ls, idxName),
				util.VTrim(ls, idxReady),
				util.VTrim(ls, idxStatus),
				util.VTrim(ls, idxRestart),
				util.VTrim(ls, idxAge),
			})
		}

		podItems = items
		wg.Done()
	}()

	wg.Add(1)
	go func() {
		command := fmt.Sprintf("kubectl top pod -n %v -l app=%v", namespace, appName)
		if namespace == "" {
			command = fmt.Sprintf("kubectl top pod -l app=%v", appName)
		}

		out, err := util.Terminal("", command)
		if err != nil {
			wg.Done()
			return
		}

		keys, vals := util.MapKV(out, "NAME", "CPU(cores)", "MEMORY(bytes)")
		var (
			idxName   = keys["NAME"]
			idxCpu    = keys["CPU(cores)"]
			idxMemory = keys["MEMORY(bytes)"]
		)

		for _, ls := range vals {
			topItems[util.VTrim(ls, idxName)] = []string{
				util.VTrim(ls, idxCpu),
				util.VTrim(ls, idxMemory),
			}
		}

		wg.Done()
	}()

	wg.Add(1)
	go func() {
		command := fmt.Sprintf("kubectl get pods -o custom-columns='NAME:.metadata.name,IMAGES:.spec.containers[*].image,CREATION:.metadata.creationTimestamp' -n %v -l app=%v", namespace, appName)
		if namespace == "" {
			command = fmt.Sprintf("kubectl get pods -o custom-columns='NAME:.metadata.name,IMAGES:.spec.containers[*].image,CREATION:.metadata.creationTimestamp' -l app=%v", appName)
		}

		out, err := util.Terminal("", command)
		if err != nil {
			wg.Done()
			return
		}

		keys, vals := util.MapKV(out, "NAME", "IMAGES", "CREATION")
		var (
			idxName     = keys["NAME"]
			idxImg      = keys["IMAGES"]
			idxCreation = keys["CREATION"]
		)

		for _, ls := range vals {
			img := strings.Split(util.VTrim(ls, idxImg), ":")
			createAt := time.Now()
			tm, err := time.Parse("2006-01-02T15:04:05Z07:00", util.VTrim(ls, idxCreation))
			if err == nil {
				createAt = tm
			}

			imgItems[util.VTrim(ls, idxName)] = []any{
				img[len(img)-1],
				createAt,
			}

		}

		wg.Done()
	}()

	wg.Wait()

	pods := make([][]any, 0)
	for i, pod := range podItems {
		var (
			podName  = pod[0].(string)
			cpu      = "-"
			mem      = "-"
			img      = "-"
			createAt = time.Now()
		)

		if vals, ok := topItems[podName]; ok {
			cpu = vals[0]
			mem = vals[1]
		}

		if vals, ok := imgItems[podName]; ok {
			img = vals[0].(string)
			createAt = vals[1].(time.Time)
		}

		pods = append(pods, []any{
			createAt,
			i + 1,
			strings.ReplaceAll(podName, appName+"-", ""),
			pod[1],
			pod[2],
			cpu,
			mem,
			pod[4],
			img,
			pod[3],
		})
	}

	sort.SliceStable(pods, func(i, j int) bool {
		x := pods[i][0].(time.Time)
		y := pods[j][0].(time.Time)
		return x.After(y)
	})

	for i := range pods {
		ls := pods[i][1:]
		ls[0] = i + 1
		pods[i] = ls
	}

	pods = slices.Insert(pods, 0, []any{"", "NAME", "READY", "STATUS", "CPU", "MEM", "AGE", "IMG", "RES"})
	output := util.Build(pods, 0)

	if len(hpaHeader) > 0 && len(hpaItems) > 0 {
		firstSpace := ""
		for len(firstSpace) != len(fmt.Sprintf("%v", pods[len(pods)-1][0])) {
			firstSpace += singleSpace
		}

		hpaHeader = slices.Insert(hpaHeader, 0, "")
		newHpaItems := make([][]any, 0)
		for _, ls := range hpaItems {
			arr := []any{firstSpace}
			arr = append(arr, ls...)
			newHpaItems = append(newHpaItems, arr)
		}

		items := make([][]any, 0)
		items = append(items, hpaHeader)
		items = append(items, newHpaItems...)

		output = util.Build(items) + "\n\n" + output
	}

	return output
}
