from PyQt6.QtCore import QPointF, QLineF

# from src.geom.point import Point
# from src.geom.vector import Vector
from PyQt6.QtCore import QPointF
from PyQt6.QtGui import QPainterPath, QFont, QTransform


def create_text_path(
    text: str, pos: QPointF, width: float, bold: bool = False
) -> QPainterPath:
    symbol = QPainterPath()
    font = QFont()
    font.setBold(bold)
    symbol.addText(0, 0, font, text)

    normalizing_transform = QTransform()
    # normalize and flip to it appears the right way up
    br = symbol.boundingRect()
    scale = min(1.0 / br.width(), 1.0 / br.height())
    normalizing_transform.scale(scale, -scale)
    # translate to center in the bounding box. The center of the text will
    # appear at the desired coordinates
    normalizing_transform.translate(
        -br.x() - br.width() / 2.0, -br.y() - br.height() / 2.0
    )
    symbol = normalizing_transform.map(symbol)

    # Transform to the desired size and location. Must be done after normalizing
    locating_transform = QTransform()
    locating_transform.translate(pos.x(), pos.y())
    locating_transform.scale(width, width)
    symbol = locating_transform.map(symbol)

    return symbol
