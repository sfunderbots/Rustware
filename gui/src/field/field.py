import pyqtgraph as pg
# from util.logger import LOG
from pyqtgraph.Qt.QtWidgets import *
from pyqtgraph.Qt import QtCore, QtGui
from PyQt6.QtCore import QPointF
# from . import FieldLayer
from src.field.field_layer import FieldLayer
# from src.visualizer.colors import BACKGROUND_COLOR
# from gui.
# from gui.colors import BACKGROUND_COLOR
from src.colors import BACKGROUND_COLOR

# from field_layer import FieldLayer
from PyQt6.QtWidgets import QWidget, QVBoxLayout
from PyQt6 import QtCore
# from ... import colors

from .raw_vision_layer import RawVisionLayer
# from .sim_control_layer import SimControlLayer
# from .filtered_vision_layer import FilteredVisionLayer
# from .trajectory_layer import TrajectoryLayer
# from .trajectory_obstacle_layer import TrajectoryObstacleLayer

# from software.thunderscope.field.field_layer import FieldLayer
# from software.thunderscope import common_widgets
# from software.py_constants import *
# from software.thunderscope.replay.replay_controls import ReplayControls


class Field(QWidget):
    """Wrapper to handle Field Layers and provide replay controls"""

    def __init__(self, max_x_range, max_y_range):
        """Initialize the field
        :param max_x_range: Maximum x range of the field
        :param max_y_range: Maximum y range of the field
        """
        QWidget.__init__(self)
        self.plot_widget = pg.PlotWidget()
        self.layout = QVBoxLayout()

        # Setup Field Plot
        self.plot_widget.setAspectLocked()
        self.plot_widget.showGrid(x=True, y=True, alpha=0.5)
        self.plot_widget.setBackground(BACKGROUND_COLOR)
        self.layout.addWidget(self.plot_widget)
        self.setFocusPolicy(QtCore.Qt.FocusPolicy.StrongFocus)

        self.max_x_range = max_x_range
        self.max_y_range = max_y_range

        # NOTE: This line has caused a lot of grief. DO NOT remove this, or you
        # will encounter a severe performance hit.
        #
        # If auto range is enabled, pyqtgraph will recalculate the range of the
        # field plot every time something changes. Since we have a lot of things
        # going on the field, this really bogs down the system.
        #
        # At the time of writing, this capped us at 30 FPS (when we were
        # trying to run at 60fps). Removing this allowed us to run at 60 FPS.
        #
        # More info here: https://groups.google.com/g/pyqtgraph/c/7dR_-3EP29k
        self.plot_widget.disableAutoRange()

        # Only call auto range once
        self.range_set = False

        self.plot_widget.setMouseTracking(True)
        self.plot_widget.scene().sigMouseMoved.connect(self.onMouseMoved)

        # Setup Field Plot Legend
        self.legend = pg.LegendItem((80, 60), offset=(70, 20))
        self.legend.setParentItem(self.plot_widget.graphicsItem())

        self.layers = []

        self.setLayout(self.layout)

        # ### Add layers ###
        # raw_vision_layer = RawVisionLayer()
        # self.add_layer("Raw Vision", raw_vision_layer)

    def onMouseMoved(self, scene_pos: QPointF):
        vb = self.plot_widget.plotItem.vb
        view_pos = vb.mapSceneToView(scene_pos)
        for layer in self.layers:
            layer.onMouseMoved(view_pos)

    def keyPressEvent(self, event):
        """Propagate keypress event to all field layers

        :param event: The event

        """
        self.plot_widget.setMouseEnabled(x=False, y=False)
        for layer in self.layers:
            if layer.isVisible():
                layer.keyPressEvent(event)

    def keyReleaseEvent(self, event):
        """Propagate keyrelease event to all field layers

        :param event: The event

        """
        self.plot_widget.setMouseEnabled(x=True, y=True)
        for layer in self.layers:
            if layer.isVisible():
                layer.keyReleaseEvent(event)

    def add_layer(self, name: str, layer: FieldLayer, visible: bool = True):
        """Add a layer to this field and to the legend.

        :param name: The name of the layer
        :param layer: The FieldLayer graphics object
        :param visible: If True, the layer will be visible on startup. If False, the layer will be hidden on startup
        """
        self.layers.append(layer)
        self.plot_widget.addItem(layer)
        self.legend.addItem(layer, name)
        if not visible:
            layer.hide()

    def refresh(self):
        """Trigger an update on all the layers"""
        for layer in self.layers:
            layer.update()

        # Set the field range once
        if not self.range_set:
            self.plot_widget.setRange(
                xRange=(-int(self.max_x_range / 2), int(self.max_x_range / 2)),
                yRange=(-int(self.max_y_range / 2), int(self.max_y_range / 2)),
            )
            self.range_set = True
