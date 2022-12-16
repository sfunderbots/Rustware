from pyqtgraph.Qt.QtWidgets import *
from queue import Queue
import queue
import pyqtgraph.console as pg_console
from src.util.logger import LOG


class LogLevelCheckboxes(QWidget):
    def __init__(self):
        """Check boxes to filter g3log levels"""
        QWidget.__init__(self)
        layout = QGridLayout()
        self.setLayout(layout)

        # Creates 4 checkboxes based on the 4 log types
        self.debug_checkbox = QCheckBox("DEBUG")
        self.debug_checkbox.setChecked(True)
        layout.addWidget(self.debug_checkbox, 0, 0)

        self.info_checkbox = QCheckBox("INFO")
        self.info_checkbox.setChecked(True)
        layout.addWidget(self.info_checkbox, 0, 1)

        self.warning_checkbox = QCheckBox("WARNING")
        self.warning_checkbox.setChecked(True)
        layout.addWidget(self.warning_checkbox, 0, 2)

        self.error_checkbox = QCheckBox("ERROR")
        self.error_checkbox.setChecked(True)
        layout.addWidget(self.error_checkbox, 0, 3)


class LogWidget(QWidget):
    def __init__(self, buffer_size=10):
        QWidget.__init__(self)

        self.console_widget = pg_console.ConsoleWidget()
        self.layout = QVBoxLayout()

        # disable input and buttons
        self.console_widget.input.hide()
        self.console_widget.ui.exceptionBtn.hide()
        self.console_widget.ui.historyBtn.hide()

        self.checkbox_widget = LogLevelCheckboxes()

        self.layout.addWidget(self.console_widget)
        self.layout.addWidget(self.checkbox_widget)
        self.setLayout(self.layout)
        self.msg_buffer = Queue(maxsize=1000)

        self.level_checkbox_map = {
            "DEBUG": self.checkbox_widget.debug_checkbox,
            "INFO": self.checkbox_widget.info_checkbox,
            "WARNING": self.checkbox_widget.warning_checkbox,
            "ERROR": self.checkbox_widget.error_checkbox,
        }

    def add_log_record_to_buffer(self, record):
        try:
            self.msg_buffer.put_nowait(record)
        except queue.Full:
            LOG.error("LogWidget unable to keep up with logs")

    def refresh(self):
        while not self.msg_buffer.empty():
            try:
                level, msg = self.msg_buffer.get_nowait()
            except queue.Empty:
                return

            if any(
                [
                    level == level_name and level_checkbox.isChecked()
                    for level_name, level_checkbox in self.level_checkbox_map.items()
                ]
            ):
                self.console_widget.write(msg + "\n")
