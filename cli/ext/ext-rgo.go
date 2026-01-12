/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package ext

import (
	"math/rand"
	"time"

	"github.com/go-vgo/robotgo"
)

func cliRgo() {
	sx, sy := robotgo.GetScreenSize()

	sx = int(float32(sx) * 0.01)
	sy = int(float32(sy) * 0.01)

	for {
		x := random(sx)
		y := random(sy)

		robotgo.MoveSmooth(x, y)
		time.Sleep(time.Millisecond * 500)
	}
}

func random(max int) int {
	min := 0
	return rand.Intn(max-min) + min
}
