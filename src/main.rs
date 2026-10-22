#[derive(Debug)]
struct MyOpen ;

#[derive(Debug)]
struct MyClose ;

#[derive(Debug)]
struct MySocket<S> {
    _s: std::marker::PhantomData<S>
}

impl MySocket<MyOpen> {
    fn close(&self) ->MySocket<MyClose> {
        MySocket { 
            _s: std::marker::PhantomData 
        }
    }
}

impl MySocket<MyClose> {
    fn open(&self) ->MySocket<MyOpen> {
        MySocket {
            _s: std::marker::PhantomData
        }
    }
}

fn main() {
    let my_cl = MySocket::<MyClose> {
        _s: std::marker::PhantomData
    } ;

    println!("closed_1: {my_cl:?}") ;
    
    /* Error:
    no method named `close` found for struct `MySocket<MyClose>` in the current scope
the method was found for `MySocket<MyOpen>`
    let my_op = my_cl.close() ;
    */

    let my_op = my_cl.open() ;

    println!("opened_1: {my_op:?}") ;

    let my_cl = my_op.close() ;

    println!("closed_2: {my_cl:?}") ;
} /* Out:
closed_1: MySocket { _s: PhantomData<o_095::MyClose> }
opened_1: MySocket { _s: PhantomData<o_095::MyOpen> }
closed_2: MySocket { _s: PhantomData<o_095::MyClose> }
*/
