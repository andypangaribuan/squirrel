/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package util

import (
	"fmt"
	"sort"
	"squirrel/model"
	"time"
)

func SortRows(rows [][]any, keys []model.OrderColumn) {
	sort.SliceStable(rows, func(i, j int) bool {
		ri, rj := rows[i], rows[j]
		for _, k := range keys {
			// safety: skip if column missing
			if k.Index >= len(ri) || k.Index >= len(rj) {
				continue
			}
			c := compare(ri[k.Index], rj[k.Index])
			if c == 0 {
				continue // tie → next key
			}
			if k.Desc {
				return c > 0
			}
			return c < 0
		}
		return false // completely equal by all keys
	})
}

func compare(a, b any) int {
	switch va := a.(type) {
	case string:
		vb, _ := b.(string)
		switch {
		case va < vb:
			return -1
		case va > vb:
			return 1
		default:
			return 0
		}

	case int:
		vb, _ := b.(int)
		switch {
		case va < vb:
			return -1
		case va > vb:
			return 1
		default:
			return 0
		}

	case int64:
		vb, _ := b.(int64)
		switch {
		case va < vb:
			return -1
		case va > vb:
			return 1
		default:
			return 0
		}

	case float64:
		vb, _ := b.(float64)
		switch {
		case va < vb:
			return -1
		case va > vb:
			return 1
		default:
			return 0
		}

	case bool:
		vb, _ := b.(bool)
		// false < true
		switch {
		case !va && vb:
			return -1
		case va && !vb:
			return 1
		default:
			return 0
		}

	case time.Time:
		vb, _ := b.(time.Time)
		if va.Before(vb) {
			return -1
		}
		if va.After(vb) {
			return 1
		}
		return 0

	default:
		// Fallback: stringify (works but less precise)
		sa := fmt.Sprint(a)
		sb := fmt.Sprint(b)
		switch {
		case sa < sb:
			return -1
		case sa > sb:
			return 1
		default:
			return 0
		}
	}
}
