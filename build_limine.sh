if [ -z "$(ls -A 'build/limine')" ]; then
    git clone https://codeberg.org/Limine/Limine.git build/limine --branch=v10.x-binary --depth=1
else
    git -C build/limine pull
fi

make -C build/limine
