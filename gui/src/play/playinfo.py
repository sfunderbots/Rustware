import logging

import pyqtgraph as pg
from abc import abstractmethod
import pyqtgraph.console as pg_console
from pyqtgraph.Qt import QtCore, QtGui
from pyqtgraph.Qt.QtWidgets import *
from src.gameplay.gameplay_data import GameplayData
from collections import defaultdict


class TableInfoWidget(QTableWidget):
    def __init__(self):
        QTableWidget.__init__(self)

        self.verticalHeader().setVisible(False)
        self.table_data = defaultdict(list)

    @abstractmethod
    def update_data(self, data: GameplayData):
        pass

    def set_data(self, data):
        """Data to set in the table
        :param data: dict containing {"column_name": [column_items]}
        """
        horizontal_headers = []

        # empirically makes even bolded items fit within columns
        HEADER_SIZE_HINT_WIDTH_EXPANSION = 12
        ITEM_SIZE_HINT_WIDTH_EXPANSION = 10

        num_rows = max([len(v) for v in data.values()])
        self.setRowCount(num_rows)
        num_cols = len(data.keys())
        self.setColumnCount(num_cols)

        for n, key in enumerate(data.keys()):
            horizontal_headers.append(key)

            for m, item in enumerate(data[key]):
                newitem = QTableWidgetItem(item)
                newitem.setSizeHint(
                    QtCore.QSize(
                        max(
                            len(key) * HEADER_SIZE_HINT_WIDTH_EXPANSION,
                            len(item) * ITEM_SIZE_HINT_WIDTH_EXPANSION,
                        ),
                        1,
                    )
                )
                self.setItem(m, n, newitem)

        self.setHorizontalHeaderLabels(horizontal_headers)

    def refresh(self):
        if self.table_data:
            self.set_data(self.table_data)

        self.resizeColumnsToContents()
        self.resizeRowsToContents()


class PlayInfoWidget(TableInfoWidget):
    def __init__(self):
        TableInfoWidget.__init__(self)

    def update_data(self, data: GameplayData):
        self.table_data.clear()
        for id, tactic in data.assigned_tactics.items():
            self.table_data["Play"].append(data.play.name())
            self.table_data["Robot ID"].append(str(id))
            self.table_data["Tactic"].append(str(tactic.__class__.__name__))
            self.table_data["Debug Info"].append(tactic.debug_info())


class MiscInfoWidget(TableInfoWidget):
    def __init__(self):
        TableInfoWidget.__init__(self)

    def update_data(self, data: GameplayData):
        self.table_data.clear()
        self.table_data["Key"].append("Friendly Possession")
        self.table_data["Value"].append(str(data.friendly_has_possession))
        self.table_data["Key"].append("GC Command")
        self.table_data["Value"].append(
            str(data.gamecontroller.game_state.command_string())
        )
