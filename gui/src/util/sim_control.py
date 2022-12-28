from third_party.ssl_simulation_protocol.ssl_simulation_control_pb2 import SimulatorControl, TeleportBall, TeleportRobot

def make_teleport_ball_command(x, y, vx, vy) -> SimulatorControl:
    teleport_ball = TeleportBall()
    teleport_ball.x = x
    teleport_ball.y = y
    teleport_ball.vx = vx
    teleport_ball.vy = vy

    simulator_control = SimulatorControl()
    simulator_control.teleport_ball.CopyFrom(teleport_ball)

    return simulator_control
