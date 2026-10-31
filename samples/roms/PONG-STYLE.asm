; PONG-STYLE: two paddles, one ball (a second ball can be switched on live), first to 9 points.
; Keys: 1 / 4 move the left paddle up / down, C / D the right paddle.
; Original program written for NecroCore (2026). Not a historical binary.
; Registers: V2 left y, V3 right y, V4/V5 ball x/y, V6/V7 ball dx/dy (0x01 or 0xFF),
;            V8/V9 scores, VA paddle height, V0/V1 scratch,
;            VB/VC ball 2 x/y, VD/VE ball 2 dx/dy (used only while BALL2_ON is 1).
start:
    CLS
    LD V2, 12
    LD V3, 12
    LD V8, 0
    LD V9, 0
    CALL reset_ball2
    .patch PADDLE_H paddle height in rows, 1 to 15
    LD VA, 4
    CALL reset_ball
loop:
    CLS
    ; left paddle input
    LD V0, 1
    SKNP V0
    ADD V2, 0xFF
    LD V0, 4
    SKNP V0
    ADD V2, 1
    ; right paddle input
    LD V0, 0xC
    SKNP V0
    ADD V3, 0xFF
    LD V0, 0xD
    SKNP V0
    ADD V3, 1
    ; clamp paddles to the screen
    SNE V2, 0xFF
    LD V2, 0
    SNE V3, 0xFF
    LD V3, 0
    SNE V2, 29
    LD V2, 28
    SNE V3, 29
    LD V3, 28
    ; move the ball
    ADD V4, V6
    ADD V5, V7
    ; bounce off the top and bottom walls
    SNE V5, 0
    LD V7, 1
    SNE V5, 31
    LD V7, 0xFF
    ; paddle collisions and scoring
    SNE V4, 1
    CALL hit_left
    SNE V4, 62
    CALL hit_right
    SNE V4, 0
    CALL point_right
    SNE V4, 63
    CALL point_left
    ; second ball: switched on by patching BALL2_ON to 1 while the game runs
    .patch BALL2_ON second ball switch, 0 off and 1 on
    LD V0, 0
    SE V0, 1
    JP ball2_done
    ADD VB, VD
    ADD VC, VE
    SNE VC, 0
    LD VE, 1
    SNE VC, 31
    LD VE, 0xFF
    SNE VB, 1
    CALL hit2_left
    SNE VB, 62
    CALL hit2_right
    SNE VB, 0
    CALL point2_right
    SNE VB, 63
    CALL point2_left
    LD I, ball_sprite
    DRW VB, VC, 1
ball2_done:
    ; draw
    LD I, paddle_sprite
    LD V0, 0
    DRW V0, V2, 4
    LD V0, 63
    DRW V0, V3, 4
    LD I, ball_sprite
    DRW V4, V5, 1
    LD F, V8
    LD V0, 24
    LD V1, 1
    DRW V0, V1, 5
    LD F, V9
    LD V0, 36
    DRW V0, V1, 5
    ; game over at 9 points
    SNE V8, 9
    JP game_over
    SNE V9, 9
    JP game_over
    ; frame delay in 60 Hz ticks: 2 is normal speed, 1 doubles the pace
    .patch FRAME_DELAY ticks to wait per frame, 2 normal and 1 double speed
    LD V0, 2
    LD DT, V0
wait:
    LD V0, DT
    SE V0, 0
    JP wait
    JP loop
hit_left:
    LD V0, V5
    SUB V0, V2
    SE VF, 1
    RET
    LD V1, VA
    ADD V1, 0xFF
    SUB V1, V0
    SE VF, 1
    RET
    LD V6, 1
    RET
hit_right:
    LD V0, V5
    SUB V0, V3
    SE VF, 1
    RET
    LD V1, VA
    ADD V1, 0xFF
    SUB V1, V0
    SE VF, 1
    RET
    LD V6, 0xFF
    RET
point_right:
    ADD V9, 1
    CALL reset_ball
    RET
point_left:
    ADD V8, 1
    CALL reset_ball
    RET
reset_ball:
    LD V4, 32
    LD V5, 16
    .patch BALL_DX ball horizontal velocity per frame (0x01 right, 0xFF left)
    LD V6, 1
    .patch BALL_DY ball vertical velocity per frame (0x01 down, 0xFF up)
    LD V7, 1
    RET
hit2_left:
    LD V0, VC
    SUB V0, V2
    SE VF, 1
    RET
    LD V1, VA
    ADD V1, 0xFF
    SUB V1, V0
    SE VF, 1
    RET
    LD VD, 1
    RET
hit2_right:
    LD V0, VC
    SUB V0, V3
    SE VF, 1
    RET
    LD V1, VA
    ADD V1, 0xFF
    SUB V1, V0
    SE VF, 1
    RET
    LD VD, 0xFF
    RET
point2_right:
    ADD V9, 1
    CALL reset_ball2
    RET
point2_left:
    ADD V8, 1
    CALL reset_ball2
    RET
reset_ball2:
    LD VB, 32
    LD VC, 8
    LD VD, 0xFF
    LD VE, 1
    RET
game_over:
    JP game_over
serve_sound:
    ; Dropped from the 1972 cabinet build: nothing calls this any more.
    LD V0, 3
    LD ST, V0
    RET
paddle_sprite:
    DB 0x80, 0x80, 0x80, 0x80
ball_sprite:
    DB 0x80, 0x00
