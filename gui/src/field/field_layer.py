import pyqtgraph as pg
from pyqtgraph.Qt import QtCore


class FieldLayer(pg.GraphicsObject):
    def __init__(self):
        pg.GraphicsObject.__init__(self)

        # options for the layer, used to configure the legend
        self.opts = {
            "pxMode": True,
            "useCache": True,
            "antialias": True,
            "name": None,
            "symbol": "o",
            "size": 7,
            "pen": pg.mkPen("w"),
            "brush": pg.mkBrush("w"),
        }

    def boundingRect(self):
        """boundingRect _must_ indicate the entire area that will be drawn on or
        else we will get artifacts and possibly crashing.

        :return: Rectangle that covers the entire field
        """
        # Size of Divison A field
        return QtCore.QRectF(-6.75, -5.25, 13.5, 10.5)

    def onMouseMoved(self, event):
        return
