from setuptools import find_packages, setup

setup(
    name="LaTeX oriented pre-commit hooks",
    description="Contains hook(s) for pre-commit, see http://pre-commit.com",
    author="Jonas Bushart <jonas@bushart.org>",
    version="1.0.0",
    classifiers=[
        "License :: OSI Approved :: Apache 2 License",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3.7",
    ],
    packages=find_packages(),
    entry_points={
        "console_scripts": ["consistent_spelling = latexhooks.consistent_spelling:main"]
    },
)
