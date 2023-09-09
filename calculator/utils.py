import enum


class PrintableEnum(enum.Enum):
    def __str__(self) -> str:
        return self.name

    __repr__ = __str__
